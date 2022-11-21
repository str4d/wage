use age::secrecy::{ExposeSecret, SecretVec};
use futures::{
    io::AsyncWrite,
    ready,
    sink::Sink,
    task::{Context, Poll},
};
use js_sys::Uint8Array;
use pin_project::pin_project;
use std::pin::Pin;
use wasm_bindgen::prelude::*;

struct Chunk {
    bytes: SecretVec<u8>,
    offset: usize,
}

/// Wraps an `age::stream::StreamWriter` in a chunked `Sink` interface.
#[pin_project(project = WriteSinkerProj)]
pub(crate) struct WriteSinker<W: AsyncWrite + Unpin> {
    #[pin]
    writer: W,
    chunk: Option<Chunk>,
}

impl<W: AsyncWrite + Unpin> WriteSinker<W> {
    pub(crate) fn new(writer: W) -> Self {
        WriteSinker {
            writer,
            chunk: None,
        }
    }

    fn poll_write_chunk(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), JsValue>> {
        let WriteSinkerProj { mut writer, chunk } = self.project();

        if let Some(chunk) = chunk.as_mut() {
            loop {
                chunk.offset += ready!(writer
                    .as_mut()
                    .poll_write(cx, &chunk.bytes.expose_secret()[chunk.offset..]))
                .map_err(|e| JsValue::from(format!("{}", e)))?;
                if chunk.offset == chunk.bytes.expose_secret().len() {
                    break;
                }
            }
        }
        *chunk = None;

        Poll::Ready(Ok(()))
    }
}

impl<W: AsyncWrite + Unpin> Sink<JsValue> for WriteSinker<W> {
    type Error = JsValue;

    fn poll_ready(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.as_mut().poll_write_chunk(cx)
    }

    fn start_send(mut self: Pin<&mut Self>, chunk: JsValue) -> Result<(), Self::Error> {
        if self.chunk.is_none() {
            self.chunk = Some(Chunk {
                bytes: SecretVec::new(Uint8Array::from(chunk).to_vec()),
                offset: 0,
            });
            Ok(())
        } else {
            Err(JsValue::from_str(
                "Called WriteSinker::start_send while not ready",
            ))
        }
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        ready!(self.as_mut().poll_write_chunk(cx))?;
        self.project()
            .writer
            .poll_flush(cx)
            .map_err(|e| JsValue::from(format!("{}", e)))
    }

    fn poll_close(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        ready!(self.as_mut().poll_write_chunk(cx))?;
        self.project()
            .writer
            .poll_close(cx)
            .map_err(|e| JsValue::from(format!("{}", e)))
    }
}

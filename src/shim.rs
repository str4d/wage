use futures::{
    io::{AsyncRead, AsyncWrite},
    ready,
    sink::Sink,
    stream::Stream,
    task::{Context, Poll},
};
use js_sys::Uint8Array;
use pin_project::pin_project;
use secrecy::{ExposeSecret, SecretVec};
use std::io;
use std::pin::Pin;
use wasm_bindgen::prelude::*;

/// Wraps an `AsyncRead` in a chunked `Stream` interface.
#[pin_project]
pub(crate) struct ReadStreamer<R: AsyncRead + Unpin> {
    #[pin]
    reader: R,
    chunk: Vec<u8>,
}

impl<R: AsyncRead + Unpin> ReadStreamer<R> {
    pub(crate) fn new(reader: R, max_chunk_size: usize) -> Self {
        ReadStreamer {
            reader,
            chunk: vec![0; max_chunk_size],
        }
    }
}

impl<R: AsyncRead + Unpin> Stream for ReadStreamer<R> {
    type Item = Result<JsValue, JsValue>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
        let mut this = self.project();

        let chunk_size = ready!(this.reader.poll_read(cx, &mut this.chunk))
            .map_err(|e| JsValue::from(format!("{}", e)))?;

        Poll::Ready(if chunk_size > 0 {
            Some(Ok(Uint8Array::from(&this.chunk[..chunk_size]).into()))
        } else {
            None
        })
    }
}

#[pin_project(project = SinkWriterProj)]
pub(crate) struct SinkWriter<S: Sink<JsValue, Error = JsValue> + Unpin> {
    #[pin]
    sink: S,
    default_chunk_size: usize,
    chunk: Vec<u8>,
}

impl<S: Sink<JsValue, Error = JsValue> + Unpin> SinkWriter<S> {
    pub(crate) fn new(sink: S, default_chunk_size: usize) -> Self {
        SinkWriter {
            sink,
            default_chunk_size,
            chunk: Vec::with_capacity(default_chunk_size),
        }
    }

    fn poll_sink_chunk(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        let SinkWriterProj {
            mut sink, chunk, ..
        } = self.project();

        if !chunk.is_empty() {
            ready!(sink.as_mut().poll_ready(cx))
                .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("JS error: {:?}", e)))?;
            sink.as_mut()
                .start_send(Uint8Array::from(&chunk[..]).into())
                .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("JS error: {:?}", e)))?;
            chunk.clear();
        }

        Poll::Ready(Ok(()))
    }
}

impl<S: Sink<JsValue, Error = JsValue> + Unpin> AsyncWrite for SinkWriter<S> {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        if self.chunk.len() >= self.default_chunk_size {
            ready!(self.as_mut().poll_sink_chunk(cx))?;
        }
        self.chunk.extend_from_slice(buf);
        Poll::Ready(Ok(buf.len()))
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        ready!(self.as_mut().poll_sink_chunk(cx))?;
        self.project()
            .sink
            .poll_flush(cx)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("JS error: {:?}", e)))
    }

    fn poll_close(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        ready!(self.as_mut().poll_sink_chunk(cx))?;
        self.project()
            .sink
            .poll_close(cx)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("JS error: {:?}", e)))
    }
}

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

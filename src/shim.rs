use futures::{
    io::AsyncRead,
    ready,
    stream::Stream,
    task::{Context, Poll},
};
use js_sys::Uint8Array;
use pin_project::pin_project;
use std::pin::Pin;
use wasm_bindgen::prelude::*;

/// Wraps an `age::stream::StreamReader` in a chunked `Stream` interface.
#[pin_project]
pub(crate) struct ReadStreamer {
    #[pin]
    reader: age::stream::StreamReader<Box<dyn AsyncRead + Unpin>>,
    chunk: Vec<u8>,
}

impl ReadStreamer {
    pub(crate) fn new(reader: age::stream::StreamReader<Box<dyn AsyncRead + Unpin>>) -> Self {
        ReadStreamer {
            reader,
            chunk: vec![0; 65536],
        }
    }
}

impl Stream for ReadStreamer {
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

use futures::{
    io::{AsyncRead, BufReader},
    ready,
    stream::Stream,
    task::{Context, Poll},
};
use js_sys::Uint8Array;
use pin_project::pin_project;
use std::io;
use std::pin::Pin;
use wasm_bindgen::prelude::*;
use wasm_streams::readable::{IntoStream, ReadableStreamDefaultReader};

#[pin_project]
pub(crate) struct StreamReader<'a> {
    #[pin]
    stream: IntoStream<'a>,
    cached_bytes: Option<Vec<u8>>,
}

impl<'a> StreamReader<'a> {
    pub(crate) fn new(reader: ReadableStreamDefaultReader<'a>) -> Self {
        StreamReader {
            stream: reader.into_stream(),
            cached_bytes: None,
        }
    }
}

impl<'a> AsyncRead for StreamReader<'a> {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context,
        buf: &mut [u8],
    ) -> Poll<Result<usize, io::Error>> {
        if self.cached_bytes.is_none() {
            match ready!(self.as_mut().project().stream.poll_next(cx)) {
                Some(Ok(value)) => {
                    self.cached_bytes = Some(Uint8Array::from(value).to_vec());
                }
                Some(Err(e)) => {
                    return Poll::Ready(Err(io::Error::new(
                        io::ErrorKind::Other,
                        format!("JS error: {:?}", e),
                    )))
                }
                None => return Poll::Ready(Ok(0)),
            }
        }

        // We are guaranteed to have cached bytes at this point.
        let mut cached = self.cached_bytes.take().unwrap();

        let read = if buf.len() < cached.len() {
            buf.copy_from_slice(&cached[..buf.len()]);
            self.cached_bytes = Some(cached.split_off(buf.len()));
            buf.len()
        } else {
            buf[..cached.len()].copy_from_slice(&cached);
            cached.len()
        };

        Poll::Ready(Ok(read))
    }
}

/// Wraps an `age::stream::StreamReader` in a chunked `Stream` interface.
#[pin_project]
pub(crate) struct ReadStreamer<'a> {
    #[pin]
    reader: age::stream::StreamReader<BufReader<StreamReader<'a>>>,
    chunk: Vec<u8>,
}

impl<'a> ReadStreamer<'a> {
    pub(crate) fn new(reader: age::stream::StreamReader<BufReader<StreamReader<'a>>>) -> Self {
        ReadStreamer {
            reader,
            chunk: vec![0; 65536],
        }
    }
}

impl<'a> Stream for ReadStreamer<'a> {
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

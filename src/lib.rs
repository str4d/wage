mod utils;

use futures::{
    io::{AsyncRead, BufReader},
    ready,
    stream::Stream,
    task::{Context, Poll},
};
use js_sys::Uint8Array;
use pin_project::pin_project;
use secrecy::SecretString;
use std::io;
use std::pin::Pin;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_streams::readable::{IntoStream, ReadableStream, ReadableStreamDefaultReader};

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
}

#[wasm_bindgen]
pub fn greet() {
    alert("Hello, wage!");
}

#[pin_project]
struct StreamReader<'a> {
    #[pin]
    stream: IntoStream<'a>,
    cached_bytes: Option<Vec<u8>>,
}

impl<'a> StreamReader<'a> {
    fn new(reader: ReadableStreamDefaultReader<'a>) -> Self {
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

struct Decryptor<'a> {
    inner: age::Decryptor<BufReader<StreamReader<'a>>>,
}

impl<'a> Decryptor<'a> {
    async fn new(reader: ReadableStreamDefaultReader<'a>) -> Result<Decryptor<'a>, JsValue> {
        let inner = age::Decryptor::new_async(StreamReader::new(reader))
            .await
            .map_err(|e| JsValue::from_str(&format!("{}", e)))?;

        Ok(Decryptor { inner })
    }

    fn is_recipients(&self) -> bool {
        match &self.inner {
            age::Decryptor::Recipients(_) => true,
            age::Decryptor::Passphrase(_) => false,
        }
    }

    fn is_passphrase(&self) -> bool {
        match &self.inner {
            age::Decryptor::Recipients(_) => false,
            age::Decryptor::Passphrase(_) => true,
        }
    }

    async fn decrypt_with_passphrase(
        self,
        passphrase: &SecretString,
    ) -> Result<age::stream::StreamReader<BufReader<StreamReader<'a>>>, JsValue> {
        let decryptor = match self.inner {
            age::Decryptor::Recipients(_) => panic!("Shouldn't be called"),
            age::Decryptor::Passphrase(d) => d,
        };

        decryptor
            .decrypt_async(passphrase, None)
            .map_err(|e| JsValue::from(format!("{}", e)))
    }
}

/// Attempts to parse the given file as an age-encrypted file, and returns a decryptor.
#[wasm_bindgen]
pub async fn decryptor_for_file(file: web_sys::File) -> Result<JsValue, JsValue> {
    // Convert from the opaque web_sys::ReadableStream Rust type to the fully-functional
    // wasm_streams::readable::ReadableStream.
    let mut stream = ReadableStream::from_raw(
        file.stream()
            .unchecked_into::<wasm_streams::readable::sys::ReadableStream>(),
    );

    let decryptor = Box::new(Decryptor::new(stream.get_reader()?).await?);

    // This is fiiiiine, we aren't allocating **that** much WASM memory...
    Ok(JsValue::from(Box::into_raw(decryptor) as u32))
}

/// Returns true if this decryptor requires identities.
#[wasm_bindgen]
pub fn decryptor_requires_identities(decryptor: u32) -> bool {
    let decryptor = unsafe { &mut *(decryptor as *mut Decryptor) };
    decryptor.is_recipients()
}

/// Returns true if this decryptor requires a passphrase.
#[wasm_bindgen]
pub fn decryptor_requires_passphrase(decryptor: u32) -> bool {
    let decryptor = unsafe { &mut *(decryptor as *mut Decryptor) };
    decryptor.is_passphrase()
}

/// Consumes the decryptor and returns the decrypted stream.
#[wasm_bindgen]
pub async fn decrypt_with_passphrase(
    decryptor: u32,
    passphrase: String,
) -> Result<JsValue, JsValue> {
    let decryptor = unsafe { Box::from_raw(decryptor as *mut Decryptor) };

    let stream = decryptor
        .decrypt_with_passphrase(&SecretString::new(passphrase))
        .await?;

    Ok(JsValue::from(Box::into_raw(Box::new(stream)) as u32))
}

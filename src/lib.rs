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

/// A newtype around a pointer to an [`age::Decryptor`].
#[wasm_bindgen]
pub struct Decryptor(u64);

impl Decryptor {
    async fn new<'a>(reader: ReadableStreamDefaultReader<'a>) -> Result<Decryptor, JsValue> {
        let inner = age::Decryptor::new_async(StreamReader::new(reader))
            .await
            .map_err(|e| JsValue::from_str(&format!("{}", e)))?;

        Ok(Decryptor(Box::into_raw(Box::new(inner)) as u64))
    }

    fn as_ref<'a>(&self) -> &age::Decryptor<BufReader<StreamReader<'a>>> {
        unsafe { &*(self.0 as *const age::Decryptor<BufReader<StreamReader<'a>>>) }
    }

    fn into_box<'a>(self) -> Box<age::Decryptor<BufReader<StreamReader<'a>>>> {
        unsafe { Box::from_raw(self.0 as *mut age::Decryptor<BufReader<StreamReader<'a>>>) }
    }
}

#[wasm_bindgen]
impl Decryptor {
    /// Returns `true` if the file was encrypted to a list of recipients, and requires
    /// identities for decryption.
    pub fn requires_identities(&self) -> bool {
        match &self.as_ref() {
            age::Decryptor::Recipients(_) => true,
            age::Decryptor::Passphrase(_) => false,
        }
    }

    /// Returns `true` if the file was encrypted to a passphrase.
    pub fn requires_passphrase(&self) -> bool {
        match &self.as_ref() {
            age::Decryptor::Recipients(_) => false,
            age::Decryptor::Passphrase(_) => true,
        }
    }

    /// Consumes the decryptor and returns the decrypted stream.
    pub async fn decrypt_with_passphrase(
        self,
        passphrase: String,
    ) -> Result<DecryptedStream, JsValue> {
        let decryptor = match *self.into_box() {
            age::Decryptor::Recipients(_) => panic!("Shouldn't be called"),
            age::Decryptor::Passphrase(d) => d,
        };

        decryptor
            .decrypt_async(&SecretString::new(passphrase), None)
            .map(DecryptedStream::from)
            .map_err(|e| JsValue::from(format!("{}", e)))
    }
}

/// A newtype around a pointer to an [`age::stream::StreamReader`].
#[wasm_bindgen]
pub struct DecryptedStream(u64);

impl<'a> From<age::stream::StreamReader<BufReader<StreamReader<'a>>>> for DecryptedStream {
    fn from(stream: age::stream::StreamReader<BufReader<StreamReader<'a>>>) -> Self {
        DecryptedStream(Box::into_raw(Box::new(stream)) as u64)
    }
}

/// Attempts to parse the given file as an age-encrypted file, and returns a decryptor.
#[wasm_bindgen]
pub async fn decryptor_for_file(file: web_sys::File) -> Result<Decryptor, JsValue> {
    // Convert from the opaque web_sys::ReadableStream Rust type to the fully-functional
    // wasm_streams::readable::ReadableStream.
    let mut stream = ReadableStream::from_raw(
        file.stream()
            .unchecked_into::<wasm_streams::readable::sys::ReadableStream>(),
    );

    let decryptor = Decryptor::new(stream.get_reader()?).await?;

    Ok(decryptor)
}

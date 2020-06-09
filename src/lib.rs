mod shim;
mod utils;

use futures::{AsyncRead, TryStreamExt};
use js_sys::Uint8Array;
use secrecy::SecretString;
use std::io;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_streams::readable::ReadableStream;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

/// Type alias to ensure consistent types across the JavaScript type erasure.
type AgeDecryptor<'a> = age::Decryptor<Box<dyn AsyncRead + Unpin + 'a>>;

/// A newtype around a pointer to an [`age::Decryptor`].
#[wasm_bindgen]
pub struct Decryptor(u64);

impl<'a> From<AgeDecryptor<'a>> for Decryptor {
    fn from(inner: AgeDecryptor<'a>) -> Self {
        Decryptor(Box::into_raw(Box::new(inner)) as u64)
    }
}

impl Decryptor {
    fn as_ref<'a>(&self) -> &AgeDecryptor<'a> {
        unsafe { &*(self.0 as *const AgeDecryptor<'a>) }
    }

    fn into_box<'a>(self) -> Box<AgeDecryptor<'a>> {
        unsafe { Box::from_raw(self.0 as *mut AgeDecryptor<'a>) }
    }
}

#[wasm_bindgen]
impl Decryptor {
    /// Attempts to parse the given file as an age-encrypted file, and returns a decryptor.
    pub async fn new(file: web_sys::File) -> Result<Decryptor, JsValue> {
        // This is an entrance from JS to our WASM APIs; perform one-time setup steps.
        utils::set_panic_hook();

        // Convert from the opaque web_sys::ReadableStream Rust type to the fully-functional
        // wasm_streams::readable::ReadableStream.
        let mut stream = ReadableStream::from_raw(file.stream().dyn_into().unwrap_throw());

        let reader: Box<dyn AsyncRead + Unpin> = Box::new(
            stream
                .get_reader()
                .into_stream()
                .map_ok(|chunk| Uint8Array::from(chunk).to_vec())
                .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("JS error: {:?}", e)))
                .into_async_read(),
        );

        let inner: AgeDecryptor = age::Decryptor::new_async(reader)
            .await
            .map_err(|e| JsValue::from_str(&format!("{}", e)))?;

        Ok(inner.into())
    }

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
    ) -> Result<wasm_streams::readable::sys::ReadableStream, JsValue> {
        let decryptor = match *self.into_box() {
            age::Decryptor::Recipients(_) => panic!("Shouldn't be called"),
            age::Decryptor::Passphrase(d) => d,
        };

        let reader = decryptor
            .decrypt_async(&SecretString::new(passphrase), None)
            .map_err(|e| JsValue::from(format!("{}", e)))?;

        Ok(ReadableStream::from_stream(shim::ReadStreamer::new(reader)).into_raw())
    }
}

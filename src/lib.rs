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

const CHUNK_SIZE: usize = 65536;

/// A newtype around an [`age::Decryptor`].
#[wasm_bindgen]
pub struct Decryptor(age::Decryptor<Box<dyn AsyncRead + Unpin>>);

#[wasm_bindgen]
pub enum DecryptorType {
    Recipients,
    Passphrase,
}

#[wasm_bindgen]
impl Decryptor {
    /// Attempts to parse the given file as an age-encrypted file, and returns a decryptor.
    pub async fn new(file: web_sys::File) -> Result<Decryptor, JsValue> {
        // This is an entrance from JS to our WASM APIs; perform one-time setup steps.
        utils::set_panic_hook();

        // Convert from the opaque web_sys::ReadableStream Rust type to the fully-functional
        // wasm_streams::readable::ReadableStream.
        // TODO: Switching from ponyfill to polyfill causes `.dyn_into().unwrap_throw()`
        // to throw, while `.unchecked_into()` works fine. I do not understand why :(
        let stream = ReadableStream::from_raw(file.stream().unchecked_into());

        let reader: Box<dyn AsyncRead + Unpin> =
            Box::new(age::armor::ArmoredReader::from_async_reader(
                stream
                    .into_stream()
                    .map_ok(|chunk| Uint8Array::from(chunk).to_vec())
                    .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("JS error: {:?}", e)))
                    .into_async_read(),
            ));

        let inner = age::Decryptor::new_async(reader)
            .await
            .map_err(|e| JsValue::from_str(&format!("{}", e)))?;

        Ok(Decryptor(inner))
    }

    /// Returns the type of this decryptor, indicating what is required to decrypt this
    /// file.
    ///
    /// - `DecryptorType::Recipients` if the file was encrypted to a list of recipients,
    ///   and requires identities for decryption.
    /// - `DecryptorType::Passphrase` if the file was encrypted to a passphrase.
    pub fn requires(&self) -> DecryptorType {
        match self.0 {
            age::Decryptor::Recipients(_) => DecryptorType::Recipients,
            age::Decryptor::Passphrase(_) => DecryptorType::Passphrase,
        }
    }

    /// Consumes the decryptor and returns the decrypted stream.
    pub async fn decrypt_with_passphrase(
        self,
        passphrase: String,
    ) -> Result<wasm_streams::readable::sys::ReadableStream, JsValue> {
        let decryptor = match self.0 {
            age::Decryptor::Recipients(_) => panic!("Shouldn't be called"),
            age::Decryptor::Passphrase(d) => d,
        };

        let reader = decryptor
            .decrypt_async(&SecretString::new(passphrase), None)
            .map_err(|e| JsValue::from(format!("{}", e)))?;

        Ok(ReadableStream::from_stream(shim::ReadStreamer::new(reader, CHUNK_SIZE)).into_raw())
    }
}

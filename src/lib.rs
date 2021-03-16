mod recipient;
mod shim;
mod utils;

use futures::{AsyncRead, TryStreamExt};
use js_sys::Uint8Array;
use secrecy::SecretString;
use std::io;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_streams::{readable::ReadableStream, writable::WritableStream};

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

const CHUNK_SIZE: usize = 65536;

/// A set of recipients to which an age file can be encrypted.
#[wasm_bindgen]
pub struct Recipients(Vec<recipient::Source>);

#[wasm_bindgen]
impl Recipients {
    /// Creates a new set containing the given recipient.
    ///
    /// Returns an error if the string is not a valid recipient.
    pub fn from_recipient(recipient: &str) -> Result<Recipients, JsValue> {
        // This is an entrance from JS to our WASM APIs; perform one-time setup steps.
        utils::set_panic_hook();
        utils::select_language();

        recipient::from_string(recipient).map(|r| Recipients(vec![r]))
    }

    /// Adds the given recipient to this set of recipients.
    ///
    /// Returns an error if the string is not a valid recipient.
    pub fn add_recipient(mut self, recipient: &str) -> Result<Recipients, JsValue> {
        self.0.push(recipient::from_string(recipient)?);
        Ok(self)
    }

    /// Merge two sets of recipients. De-duplication is not performed.
    pub fn merge(mut self, other: Recipients) -> Recipients {
        self.0.extend(other.0);
        self
    }

    /// Returns an `Encryptor` that will create an age file encrypted to the list of
    /// recipients.
    pub fn into_encryptor(self) -> Encryptor {
        let mut recipients: Vec<_> = self
            .0
            .into_iter()
            .map(|s| match s {
                recipient::Source::String(k) => vec![k],
            })
            .flatten()
            .collect();
        recipients.sort_unstable();
        recipients.dedup();

        Encryptor(age::Encryptor::with_recipients(
            recipients
                .into_iter()
                .map(|k| match k {
                    recipient::Kind::Native(r) => Box::new(r) as Box<dyn age::Recipient>,
                })
                .collect(),
        ))
    }
}

/// A newtype around an [`age::Encryptor`].
#[wasm_bindgen]
pub struct Encryptor(age::Encryptor);

#[wasm_bindgen]
impl Encryptor {
    /// Returns an `Encryptor` that will create an age file encrypted with a passphrase.
    ///
    /// This API should only be used with a passphrase that was provided by (or generated
    /// for) a human. For programmatic use cases, instead generate a `SecretKey` and then
    /// use `Encryptor::with_recipients`.
    pub fn with_user_passphrase(passphrase: String) -> Encryptor {
        // This is an entrance from JS to our WASM APIs; perform one-time setup steps.
        utils::set_panic_hook();
        utils::select_language();

        Encryptor(age::Encryptor::with_user_passphrase(SecretString::new(
            passphrase,
        )))
    }

    /// Creates a wrapper around a writer that will encrypt its input.
    ///
    /// Returns errors from the underlying writer while writing the header.
    pub async fn wrap_output(
        self,
        output: wasm_streams::writable::sys::WritableStream,
    ) -> Result<wasm_streams::writable::sys::WritableStream, JsValue> {
        // Convert from the opaque web_sys::WritableStream Rust type to the fully-functional
        // wasm_streams::writable::WritableStream.
        let sink = WritableStream::from_raw(output).into_sink();

        let writer = self
            .0
            .wrap_async_output(shim::SinkWriter::new(sink, CHUNK_SIZE))
            .await
            .map_err(|e| JsValue::from(format!("{}", e)))?;

        Ok(WritableStream::from_sink(shim::WriteSinker::new(writer)).into_raw())
    }
}

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
        utils::select_language();

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

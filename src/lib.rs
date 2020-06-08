mod shim;
mod utils;

use futures::{io::BufReader, stream::Stream};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_streams::readable::ReadableStream;

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

/// A newtype around a pointer to an [`age::Decryptor`].
#[wasm_bindgen]
pub struct Decryptor(u64);

impl Decryptor {
    fn into_box<'a>(self) -> Box<age::Decryptor<BufReader<shim::StreamReader<'a>>>> {
        unsafe { Box::from_raw(self.0 as *mut age::Decryptor<BufReader<shim::StreamReader<'a>>>) }
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
        let mut stream = ReadableStream::from_raw(
            file.stream()
                .unchecked_into::<wasm_streams::readable::sys::ReadableStream>(),
        );

        let reader = shim::StreamReader::new(stream.get_reader()?);

        let inner = age::Decryptor::new_async(reader)
            .await
            .map_err(|e| JsValue::from_str(&format!("{}", e)))?;

        Ok(Decryptor(Box::into_raw(Box::new(inner)) as u64))
    }

    /// Returns `true` if the file was encrypted to a list of recipients, and requires
    /// identities for decryption.
    pub fn requires_identities(&self) -> bool {
        false
    }

    /// Returns `true` if the file was encrypted to a passphrase.
    pub fn requires_passphrase(&self) -> bool {
        true
    }

    /// Consumes the decryptor and returns the decrypted stream.
    pub async fn decrypt_with_passphrase(
        self,
    ) -> Result<wasm_streams::readable::sys::ReadableStream, JsValue> {
        let decryptor = self.into_box();

        let reader = decryptor
            .decrypt_async()
            .map_err(|e| JsValue::from(format!("{}", e)))?;

        let stream: Box<dyn Stream<Item = Result<JsValue, JsValue>>> =
            Box::new(shim::ReadStreamer::new(reader));

        Ok(ReadableStream::from(stream).into_raw())
    }
}

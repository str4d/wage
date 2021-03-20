use std::io;

use age::x25519;
use futures::{io::BufReader, AsyncBufReadExt, StreamExt, TryStreamExt};
use js_sys::Uint8Array;
use wasm_bindgen::{JsCast, JsValue};
use wasm_streams::ReadableStream;

fn parse_identity(s: &str) -> Result<Box<dyn age::Identity>, JsValue> {
    if let Ok(sk) = s.parse::<x25519::Identity>() {
        Ok(Box::new(sk))
    } else {
        Err(JsValue::from(
            "String does not contain a supported identity",
        ))
    }
}

/// Reads file contents as a list of identities
pub(crate) async fn read_identities_list(
    file: web_sys::File,
) -> Result<Vec<Box<dyn age::Identity>>, JsValue> {
    // Convert from the opaque web_sys::ReadableStream Rust type to the fully-functional
    // wasm_streams::readable::ReadableStream.
    // TODO: Switching from ponyfill to polyfill causes `.dyn_into().unwrap_throw()`
    // to throw, while `.unchecked_into()` works fine. I do not understand why :(
    let stream = ReadableStream::from_raw(file.stream().unchecked_into());

    let buf = BufReader::new(
        stream
            .into_stream()
            .map_ok(|chunk| Uint8Array::from(chunk).to_vec())
            .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("JS error: {:?}", e)))
            .into_async_read(),
    );

    let mut lines = buf.lines().enumerate();

    let mut identities = vec![];
    while let Some((line_number, line)) = lines.next().await {
        let line = line.map_err(|e| JsValue::from(format!("{}", e)))?;

        // Skip empty lines and comments
        if line.is_empty() || line.find('#') == Some(0) {
            continue;
        }

        match parse_identity(&line) {
            Ok(kind) => identities.push(kind),
            Err(_) => {
                // Return a line number in place of the line, so we don't leak the file
                // contents in error messages.
                return Err(JsValue::from(format!(
                    "identities file {} contains non-identity data on line {}",
                    file.name(),
                    line_number + 1
                )));
            }
        }
    }

    Ok(identities)
}

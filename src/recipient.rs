use std::io;

use age::x25519;
use futures::{io::BufReader, AsyncBufReadExt, StreamExt, TryStreamExt};
use js_sys::Uint8Array;
use wasm_bindgen::{JsCast, JsValue};
use wasm_streams::ReadableStream;

pub(crate) enum Source {
    File { recipients: Vec<Kind> },
    String(Kind),
}

pub(crate) enum Kind {
    Native(x25519::Recipient),
}

impl PartialEq for Kind {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Kind::Native(a), Kind::Native(b)) => a.to_string().eq(&b.to_string()),
        }
    }
}

impl Eq for Kind {}

impl PartialOrd for Kind {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Kind {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match (self, other) {
            (Kind::Native(a), Kind::Native(b)) => a.to_string().cmp(&b.to_string()),
        }
    }
}

fn parse_recipient(s: &str) -> Result<Kind, JsValue> {
    if let Ok(pk) = s.parse::<x25519::Recipient>() {
        Ok(Kind::Native(pk))
    } else {
        Err(JsValue::from(
            "String does not contain a supported recipient",
        ))
    }
}

/// Parses a recipient from a string.
pub(crate) fn from_string(s: &str) -> Result<Source, JsValue> {
    parse_recipient(s).map(Source::String)
}

/// Reads file contents as a list of recipients
pub(crate) async fn read_recipients_list(file: web_sys::File) -> Result<Source, JsValue> {
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

    let mut recipients = vec![];
    while let Some((line_number, line)) = lines.next().await {
        let line = line.map_err(|e| JsValue::from(format!("{}", e)))?;

        // Skip empty lines and comments
        if line.is_empty() || line.find('#') == Some(0) {
            continue;
        }

        match parse_recipient(&line) {
            Ok(kind) => recipients.push(kind),
            Err(_) => {
                // Return a line number in place of the line, so we don't leak the file
                // contents in error messages.
                return Err(JsValue::from(format!(
                    "recipients file {} contains non-recipient data on line {}",
                    file.name(),
                    line_number + 1
                )));
            }
        }
    }

    Ok(Source::File { recipients })
}

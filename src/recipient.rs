use age::x25519;
use wasm_bindgen::JsValue;

pub(crate) enum Source {
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

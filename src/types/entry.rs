use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

/// Content TypeSafe.ai accepts: plain text, a JSON object, or a JSON array
///
/// Every description slot on the wire is one of these: `state`, every
/// `instructions`, each Choice option, each Score level, both Noul sides,
/// and the `legend` that comes back on a Score answer.
///
/// The API reads text, not bare numbers or bools, so `From<bool>` and
/// `From<f64>` turn those into strings before they're sent.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Entry {
  /// Plain text, e.g. `"Calm"`,
  Text(String),
  /// Named parts, e.g. `{ "what": "...", "examples": [...]}`.
  Object(Map<String, Value>),
  /// A list, e.g. the messages in a conversation.
  Array(Vec<Value>),
}

/// Borrowed text, e.g. `Entry::from("Calm")`. Copies it into a `String`.
impl From<&str> for Entry {
  fn from(text: &str) -> Self {
    Entry::Text(text.to_owned())
  }
}

/// Owned text. Moves the `String` in with no copy.
impl From<String> for Entry {
  fn from(text: String) -> Self {
    Entry::Text(text)
  }
}

// A borrowed `String`, so `&name` works without `.clone()` at the call site.
impl From<&String> for Entry {
  fn from(text: &String) -> Self {
    Entry::Text(text.clone())
  }
}

/// A bool as text: `true` becomes `"true"`.
impl From<bool> for Entry {
  fn from(flag: bool) -> Self {
    Entry::Text(flag.to_string())
  }
}

/// A number as text: `4.5` becomes `"4.5"`, and `3.0` becomes `"3"`.
impl From<f64> for Entry {
  fn from(number: f64) -> Self {
    Entry::Text(number.to_string())
  }
}

/// Lets users pass `json!(...)` directly.
impl From<Value> for Entry {
  fn from(value: Value) -> Self {
    match value {
      | Value::String(text) => Entry::Text(text),
      | Value::Object(map) => Entry::Object(map),
      | Value::Array(items) => Entry::Array(items),
      | other => Entry::Text(other.to_string()),
    }
  }
}

pub type Entries = Vec<Entry>;

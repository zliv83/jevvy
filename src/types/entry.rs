use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Content TypeSafe.ai accepts: plain text, a JSON object, or a JSON array
///
/// Type to match the shape of all `Instructions`, `Score` levels,
/// `Choice` options, and `NoulCriteria`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Entry {
  Text(String),
  Object(IndexMap<String, Value>),
  Array(Vec<Value>),
}

impl From<&str> for Entry {
  fn from(text: &str) -> Self {
    Entry::Text(text.to_owned())
  }
}

impl From<String> for Entry {
  fn from(text: String) -> Self {
    Entry::Text(text)
  }
}

impl From<&String> for Entry {
  fn from(text: &String) -> Self {
    Entry::Text(text.clone())
  }
}

/// Lets users pass `json!(...)` directly.
impl From<Value> for Entry {
  fn from(value: Value) -> Self {
    match value {
      | Value::String(text) => Entry::Text(text),
      | Value::Object(map) => Entry::Object(
        map
          .into_iter()
          .collect(),
      ),
      | Value::Array(items) => Entry::Array(items),
      | other => Entry::Text(other.to_string()),
    }
  }
}

pub type Entries = Vec<Entry>;

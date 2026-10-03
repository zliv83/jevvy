use jevvy::{Question, Reply};
use serde_json::{json, Value};
use std::{error::Error, fs, path::Path};

/// Builds a test reply to `question` using the answer from
/// `tests/fixtures/<fixture>.json`.
///
/// Question IDs come from a global counter and aren't known in advance.
/// The fixture stores only the answer. This helper wraps it in a reply
/// using the question's ID.
///
/// Add `mod common;` to any integration test file to use this helper.
pub fn reply_to(question: &impl Question, fixture: &str) -> Result<Reply, Box<dyn Error>> {
  let path = Path::new(env!("CARGO_MANIFEST_DIR"))
    .join("tests/fixtures")
    .join(format!("{fixture}.json"));
  let answer: Value = serde_json::from_str(&fs::read_to_string(path)?)?;

  let reply = json!({
    "model": "jev-fixture",
    "answers": { question.id().to_string(): answer },
    "usage": { "input_tokens": 42, "output_tokens": 8 },
  });

  Ok(serde_json::from_value(reply)?)
}

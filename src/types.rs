use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Value;
use std::{
  collections::HashMap,
  fmt,
  fmt::Display,
  sync::atomic::{AtomicU64, Ordering},
};

#[derive(Clone, Debug, Serialize)]
#[serde(untagged)]
/// Text, an object, or an array used as state, instructions, or criteria.
pub enum InputContent {
  Text(String),
  Object(HashMap<String, Value>),
  Array(Vec<Value>),
}

impl From<&str> for InputContent {
  fn from(value: &str) -> Self {
    Self::Text(value.to_owned())
  }
}

impl From<String> for InputContent {
  fn from(value: String) -> Self {
    Self::Text(value)
  }
}

// ---- State ----
#[derive(Debug, Serialize)]
#[serde(transparent)]
pub struct State(InputContent);

impl State {
  #[must_use]
  pub fn new(content: impl Into<InputContent>) -> Self {
    Self(content.into())
  }

  #[must_use]
  pub const fn content(&self) -> &InputContent {
    &self.0
  }
}

impl<T: Into<InputContent>> From<T> for State {
  fn from(value: T) -> Self {
    Self::new(value)
  }
}

// ----- Instructions ----
#[derive(Clone, Debug, Serialize)]
#[serde(transparent)]
pub struct Instructions(InputContent);

impl Instructions {
  #[must_use]
  pub fn new(content: impl Into<InputContent>) -> Self {
    Self(content.into())
  }

  #[must_use]
  pub const fn content(&self) -> &InputContent {
    &self.0
  }
}

impl<T: Into<InputContent>> From<T> for Instructions {
  fn from(value: T) -> Self {
    Self::new(value)
  }
}

// ---- Probabilities ----
#[derive(Debug, Deserialize)]
#[serde(transparent)]
pub struct Probabilities(HashMap<String, f64>);

impl Probabilities {
  pub fn iter(&self) -> impl Iterator<Item = (&str, f64)> {
    self
      .0
      .iter()
      .map(|(option, probability)| (option.as_str(), *probability))
  }
}

// ---- Confidence ----
#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(transparent)]
pub struct Confidence(f64);

impl Confidence {
  #[must_use]
  pub const fn value(self) -> f64 {
    self.0
  }
}

impl Display for Confidence {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    Display::fmt(&self.0, f)
  }
}

// ---- QuestionId ----
/// A question's ID, used as its key in the request and to find its answer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct QuestionId(u64);

impl QuestionId {
  pub(crate) fn generate() -> Self {
    static COUNTER: AtomicU64 = AtomicU64::new(0);

    let id = COUNTER.fetch_add(1, Ordering::Relaxed);

    Self(id)
  }

  fn parse(key: &str) -> Option<Self> {
    let stripped = key.strip_prefix('q')?;
    let parsed = stripped
      .parse()
      .ok()?;

    Some(Self(parsed))
  }
}

impl fmt::Display for QuestionId {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "q{}", self.0)
  }
}

impl Serialize for QuestionId {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: Serializer,
  {
    serializer.collect_str(self)
  }
}

impl<'de> Deserialize<'de> for QuestionId {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: Deserializer<'de>,
  {
    let key = String::deserialize(deserializer)?;
    Self::parse(&key).ok_or_else(|| de::Error::custom(format!("invalid question id `{key}`")))
  }
}

#[cfg(test)]
mod tests {
  use super::QuestionId;

  #[test]
  fn generated_ids_are_unique_and_display_with_prefix() {
    let a = QuestionId::generate();
    let b = QuestionId::generate();

    assert_ne!(a, b);
    assert!(a
      .to_string()
      .starts_with('q'));
  }

  #[test]
  fn question_id_round_trips_through_json() -> Result<(), serde_json::Error> {
    let id = QuestionId::generate();

    let json = serde_json::to_string(&id)?;
    let decoded: QuestionId = serde_json::from_str(&json)?;

    assert_eq!(decoded, id);
    Ok(())
  }

  #[test]
  fn question_id_rejects_keys_without_the_q_prefix_format() {
    assert!(serde_json::from_str::<QuestionId>("\"needs_attention\"").is_err());
    assert!(serde_json::from_str::<QuestionId>("\"q\"").is_err());
    assert!(serde_json::from_str::<QuestionId>("\"7\"").is_err());
  }
}

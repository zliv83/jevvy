use serde::{Deserialize, Serialize};

use crate::{
  sheet::AnswerSheet,
  types::{
    answers::{Answer, Answers, ChoiceAnswer, ScoreAnswer},
    jevvy_error::JevvyError,
  },
};

/// One reply. The model that answered, one answer per question id,
/// and the tokens used. Keep the whole thing - it's the receipt.
///
/// Named `JevvyResponse` so it's never mistaken for `reqwest::Response`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JevvyResponse {
  /// Model - The model that performed the eval
  pub model: String,

  /// The answers
  pub answers: Answers,

  /// Token usage for the request
  pub usage: Usage,
}

impl JevvyResponse {
  /// The root answer sheet.
  /// Read answers by key, or through a form.
  #[must_use]
  pub fn sheet(&self) -> AnswerSheet<'_> {
    AnswerSheet::root(&self.answers)
  }
}

/// Builds the "wrong kind of answer" error.
fn wrong_type(key: &str, expected: &'static str, found: &Answer) -> JevvyError {
  JevvyError::WrongAnswerType {
    key: key.to_owned(),
    expected,
    found: found.kind(),
  }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Usage {
  pub input_tokens:  u32,
  pub output_tokens: u32,
}

use serde::{Deserialize, Serialize};

use crate::types::{
	answers::{Answer, Answers, ChoiceAnswer, ScoreAnswer},
	error::JevvyError,
};

/// One answer per question, returned under the same ids you provided

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Response {
  /// Model - The model that performed the eval
  pub model: String,

  /// The answers
  pub answers: Answers,

  /// Token usage for the request
  pub usage: Usage,
}

impl Response {
  /// Looks up any answer by key.
  pub fn answer(&self, key: &str) -> Result<&Answer, JevvyError> {
    self
      .answers
      .get(key)
      .ok_or_else(|| JevvyError::MissingAnswer(key.to_owned()))
  }

  /// The probability of "yes" for the Noul under `key`.
  pub fn noul(&self, key: &str) -> Result<f64, JevvyError> {
    match self.answer(key)? {
      | Answer::Noul(a) => Ok(a.noul),
      | other => Err(wrong_type(key, "noul", other)),
    }
  }

  /// The Choice answer under `key`.
  pub fn choice(&self, key: &str) -> Result<&ChoiceAnswer, JevvyError> {
    match self.answer(key)? {
      | Answer::Choice(a) => Ok(a),
      | other => Err(wrong_type(key, "choice", other)),
    }
  }

  /// The Score answer under `key`.
  pub fn score(&self, key: &str) -> Result<&ScoreAnswer, JevvyError> {
    match self.answer(key)? {
      | Answer::Score(a) => Ok(a),
      | other => Err(wrong_type(key, "score", other)),
    }
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

use std::collections::HashMap;

use serde::Deserialize;

use crate::{
  choice::ChoiceAnswer, error::JevvyError, noul::NoulAnswer, questions::Question,
  score::ScoreAnswer, types::QuestionId,
};

/// A Noul, Choice, or Score answer.
///
/// Jevvy matches the answer's `"type"` to a variant name in lowercase.
/// Renaming a variant changes which answers Jevvy can decode.
#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum AnyAnswer {
  Noul(NoulAnswer),
  Choice(ChoiceAnswer),
  Score(ScoreAnswer),
}

/// Jev's reply to `jev.ask(&questions)`.
///
/// Contains an answer for each question, the model name, and token counts.
#[derive(Debug, Deserialize)]
pub struct Reply {
  model:   String,
  answers: HashMap<QuestionId, AnyAnswer>,
  usage:   Usage,
}

impl Reply {
  /// Finds the answer to `question` by ID and borrows it as `Q::Answer`.
  ///
  /// Returns `MissingAnswer` if the reply has no answer for this question,
  /// or `WrongAnswerKind` if the answer's kind doesn't match the question type.
  pub fn answer<Q: Question>(&self, question: &Q) -> Result<&Q::Answer, JevvyError> {
    // Look up the answer by ID, then extract the expected answer type.
    let id = question.id();

    let any_answer = self
      .answers
      .get(&id)
      .ok_or(JevvyError::MissingAnswer { id })?;

    Q::extract(any_answer).ok_or(JevvyError::WrongAnswerKind { id })
  }

  /// The model name reported by Jev, such as `jev-1.13.0`.
  #[must_use]
  pub fn model(&self) -> &str {
    &self.model
  }

  #[must_use]
  pub const fn usage(&self) -> Usage {
    self.usage
  }
}

/// Token counts reported for an evaluation.
#[derive(Clone, Copy, Debug, Deserialize)]
pub struct Usage {
  input_tokens:  u64,
  output_tokens: u64,
}

impl Usage {
  #[must_use]
  pub const fn input_tokens(&self) -> u64 {
    self.input_tokens
  }

  #[must_use]
  pub const fn output_tokens(&self) -> u64 {
    self.output_tokens
  }
}

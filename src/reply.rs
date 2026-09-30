use std::collections::HashMap;

use serde::Deserialize;

use crate::{
  choice::ChoiceAnswer, error::JevvyError, noul::NoulAnswer, questions::Question,
  score::ScoreAnswer, types::QuestionId,
};

/// Any kind of answer, in its wire shape. The `"type"` field picks the variant,
/// so renaming a variant changes what Jevvy can decode.
#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum AnyAnswer {
  Noul(NoulAnswer),
  Choice(ChoiceAnswer),
  Score(ScoreAnswer),
}

/// Everything Jev sends back from `jev.ask(&questions)`: one answer per
/// question, plus the model that answered and the tokens it used.
#[derive(Debug, Deserialize)]
pub struct Reply {
  model:   String,
  answers: HashMap<QuestionId, AnyAnswer>,
  usage:   Usage,
}

impl Reply {
  /// Jev's answer to `question`, as the answer type that question declares.
  ///
  /// Fails with `MissingAnswer` if this reply has no answer for the question,
  /// or `WrongAnswerKind` if the answer isn't the kind the question asked for.
  pub fn answer<Q: Question>(&self, question: &Q) -> Result<&Q::Answer, JevvyError> {
    // `Q::extract` to turn the `AnyAnswer` into `&Q::Answer`.
    let id = question.id();

    let any_answer = self
      .answers
      .get(&id)
      .ok_or(JevvyError::MissingAnswer { id })?;

    Q::extract(any_answer).ok_or(JevvyError::WrongAnswerKind { id })
  }

  /// The model that answered, as Jev reported it (e.g. `jev-1.13.0`).
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

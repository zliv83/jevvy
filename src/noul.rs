use std::fmt::Display;

use serde::{Deserialize, Serialize};

use crate::{
  error::JevvyError,
  questions::Question,
  reply::AnyAnswer,
  types::{Confidence, InputContent, Instructions, QuestionId},
};

#[derive(Clone, Debug, Serialize)]
/// A yes/no question. Returns the probability the answer is yes.
///
/// Construct one with `NoulQuestion::builder()`.
pub struct NoulQuestion {
  #[serde(skip)]
  id:           QuestionId,
  /// The yes/no question to evaluate. An object can hold the question in one
  /// field and data it refers to in the others.
  instructions: Instructions,
  /// See `NoulCriteria`.
  #[serde(skip_serializing_if = "Option::is_none")]
  criteria:     Option<NoulCriteria>,
}

impl NoulQuestion {
  #[must_use]
  pub const fn builder() -> NoulBuilder {
    NoulBuilder::new()
  }

  #[must_use]
  pub const fn instructions(&self) -> &Instructions {
    &self.instructions
  }

  #[must_use]
  pub const fn criteria(&self) -> Option<&NoulCriteria> {
    self
      .criteria
      .as_ref()
  }
}

impl Question for NoulQuestion {
  type Answer = NoulAnswer;

  fn id(&self) -> QuestionId {
    self.id
  }

  fn extract(answer: &AnyAnswer) -> Option<&Self::Answer> {
    match answer {
      | AnyAnswer::Noul(noul) => Some(noul),
      | _ => None,
    }
  }
}

#[derive(Clone, Debug, Serialize)]
/// Optional descriptions of what a yes and a no mean.
pub struct NoulCriteria {
  /// What a yes (value near 1) means.
  #[serde(rename = "true")]
  yes: InputContent,
  /// What a no (value near 0) means.
  #[serde(rename = "false")]
  no:  InputContent,
}

impl NoulCriteria {
  #[must_use]
  pub const fn yes(&self) -> &InputContent {
    &self.yes
  }

  #[must_use]
  pub const fn no(&self) -> &InputContent {
    &self.no
  }
}

#[derive(Debug)]
pub struct NoulBuilder {
  instructions: Option<Instructions>,
  criteria:     Option<NoulCriteria>,
}

impl NoulBuilder {
  /// Entry point is `NoulQuestion::builder()`.
  const fn new() -> Self {
    Self {
      instructions: None,
      criteria:     None,
    }
  }

  #[must_use]
  pub fn instructions(mut self, instructions: impl Into<Instructions>) -> Self {
    self.instructions = Some(instructions.into());
    self
  }

  #[must_use]
  pub fn criteria(mut self, yes: impl Into<InputContent>, no: impl Into<InputContent>) -> Self {
    self.criteria = Some(NoulCriteria {
      yes: yes.into(),
      no:  no.into(),
    });
    self
  }

  pub fn build(self) -> Result<NoulQuestion, JevvyError> {
    let instructions = self
      .instructions
      .ok_or(JevvyError::MissingInstructions)?;

    Ok(NoulQuestion {
      id: QuestionId::generate(),
      instructions,
      criteria: self.criteria,
    })
  }
}

/// Jev's answer to a `NoulQuestion`.
#[derive(Debug, Deserialize)]
pub struct NoulAnswer {
  noul: Confidence,
}

impl NoulAnswer {
  /// Returns the unchanged Noul value.
  ///
  /// The yes/no answer on a scale from 0 (no) to 1 (yes).
  #[must_use]
  pub const fn confidence(&self) -> Confidence {
    self.noul
  }
}

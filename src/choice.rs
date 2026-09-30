use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{
  error::JevvyError,
  questions::Question,
  reply::AnyAnswer,
  types::{Confidence, InputContent, Instructions, Probabilities, QuestionId},
};

#[derive(Clone, Debug, Serialize)]
/// Picks one option from a set you define. Returns the chosen
/// option and the full probability distribution.
///
/// Construct one with `ChoiceQuestion::builder()`.
pub struct ChoiceQuestion {
  #[serde(skip)]
  id:           QuestionId,
  /// What the model should decide. An object can hold the question
  /// in one field and data it refers to in the others.
  instructions: Instructions,
  /// A map of option to rubric description; use None when an option needs no
  /// extra detail. You can have a max of 255 options.
  criteria:     ChoiceCriteria,
}

impl ChoiceQuestion {
  #[must_use]
  pub fn builder() -> ChoiceBuilder {
    ChoiceBuilder::new()
  }

  #[must_use]
  pub const fn instructions(&self) -> &Instructions {
    &self.instructions
  }

  #[must_use]
  pub const fn criteria(&self) -> &ChoiceCriteria {
    &self.criteria
  }
}

impl Question for ChoiceQuestion {
  type Answer = ChoiceAnswer;

  fn id(&self) -> QuestionId {
    self.id
  }

  fn extract(answer: &AnyAnswer) -> Option<&Self::Answer> {
    match answer {
      | AnyAnswer::Choice(choice) => Some(choice),
      | _ => None,
    }
  }
}

#[derive(Clone, Debug, Default, Serialize)]
#[serde(transparent)]
/// The options for a choice. Keyed by option name. Serializes as a JSON object.
pub struct ChoiceCriteria(HashMap<String, Option<InputContent>>);

impl ChoiceCriteria {
  /// Each option's name, and it's description, if it has one.
  pub fn iter(&self) -> impl Iterator<Item = (&str, Option<&InputContent>)> {
    self
      .0
      .iter()
      .map(|(name, description)| (name.as_str(), description.as_ref()))
  }

  #[must_use]
  pub fn len(&self) -> usize {
    self
      .0
      .len()
  }

  #[must_use]
  pub fn is_empty(&self) -> bool {
    self
      .0
      .is_empty()
  }

  fn insert(&mut self, name: impl Into<String>, description: Option<InputContent>) {
    self
      .0
      .insert(name.into(), description);
  }
}

#[derive(Debug)]
pub struct ChoiceBuilder {
  instructions: Option<Instructions>,
  criteria:     ChoiceCriteria,
}

impl ChoiceBuilder {
  /// Entry point is `ChoiceQuestion::builder()`.
  fn new() -> Self {
    Self {
      instructions: None,
      criteria:     ChoiceCriteria::default(),
    }
  }

  #[must_use]
  pub fn instructions(mut self, instructions: impl Into<Instructions>) -> Self {
    self.instructions = Some(instructions.into());
    self
  }

  #[must_use]
  pub fn option(mut self, name: impl Into<String>, description: impl Into<InputContent>) -> Self {
    self
      .criteria
      .insert(name, Some(description.into()));
    self
  }

  #[must_use]
  pub fn option_without_description(mut self, name: impl Into<String>) -> Self {
    self
      .criteria
      .insert(name, None);
    self
  }

  pub fn build(self) -> Result<ChoiceQuestion, JevvyError> {
    let instructions = self
      .instructions
      .ok_or(JevvyError::MissingInstructions)?;

    let count = self
      .criteria
      .len();

    if !(1..=255).contains(&count) {
      return Err(JevvyError::InvalidChoiceOptionCount { count });
    }

    Ok(ChoiceQuestion {
      id: QuestionId::generate(),
      instructions,
      criteria: self.criteria,
    })
  }
}

/// Jev's answer to a `ChoiceQuestion`.
#[derive(Debug, Deserialize)]
pub struct ChoiceAnswer {
  /// The highest-probability option.
  choice:        String,
  /// Every option mapped to its probability (floats that sum to 1).
  probabilities: Probabilities,
  /// How certain the model is, derived from probabilities.
  confidence:    Confidence,
}

impl ChoiceAnswer {
  /// The highest-probability option.
  #[must_use]
  pub fn choice(&self) -> &str {
    &self.choice
  }

  /// Every option mapped to its probability (floats that sum to 1).
  #[must_use]
  pub const fn probabilities(&self) -> &Probabilities {
    &self.probabilities
  }

  /// How certain the model is, derived from probabilities.
  #[must_use]
  pub const fn confidence(&self) -> Confidence {
    self.confidence
  }
}

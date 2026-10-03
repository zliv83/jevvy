use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{
  error::JevvyError,
  questions::Question,
  reply::AnyAnswer,
  types::{Confidence, InputContent, Instructions, Probabilities, QuestionId},
};

#[derive(Clone, Debug, Serialize)]
/// A question with options you define. Jev picks one and returns
/// the probability of each option.
///
/// Build one with `ChoiceQuestion::builder()`.
pub struct ChoiceQuestion {
  #[serde(skip)]
  id:           QuestionId,
  /// What the model should decide. Use an object to include the question
  /// in one field and supporting data in others.
  instructions: Instructions,
  /// Options and their descriptions, keyed by option name.
  /// Use `None` when an option needs no description. The limit is 255 options.
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
/// The options Jev can choose from.
///
/// Serializes as a JSON object keyed by option name.
pub struct ChoiceCriteria(HashMap<String, Option<InputContent>>);

impl ChoiceCriteria {
  /// Yields each option's name and its description, if any.
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
  /// Creates an empty builder for `ChoiceQuestion::builder()`.
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
  /// The option with the highest probability.
  choice:        String,
  /// The probability of each option, keyed by name. The values sum to 1.
  probabilities: Probabilities,
  /// The model's certainty, calculated from the probabilities.
  confidence:    Confidence,
}

impl ChoiceAnswer {
  /// The option with the highest probability.
  #[must_use]
  pub fn choice(&self) -> &str {
    &self.choice
  }

  /// The probability of each option, keyed by name. The values sum to 1.
  #[must_use]
  pub const fn probabilities(&self) -> &Probabilities {
    &self.probabilities
  }

  /// The model's certainty, calculated from the probabilities.
  #[must_use]
  pub const fn confidence(&self) -> Confidence {
    self.confidence
  }
}

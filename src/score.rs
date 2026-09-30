use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{
  error::JevvyError,
  questions::Question,
  reply::AnyAnswer,
  types::{Confidence, InputContent, Instructions, Probabilities, QuestionId},
};

#[derive(Clone, Debug, Serialize)]
/// Rates the state along a rubric you define. Returns a probability-weighted
/// value across your levels.
///
/// Construct one with `ScoreQuestion::builder()`.
pub struct ScoreQuestion {
  #[serde(skip)]
  id:           QuestionId,
  /// What the model should rate. An object can hold the question in one field
  /// and data it refers to in others.
  instructions: Instructions,
  /// An ordered array of level descriptions. A Score should have at least
  /// two levels - the API accepts up to 10.
  criteria:     ScoreCriteria,
}

impl ScoreQuestion {
  #[must_use]
  pub fn builder() -> ScoreBuilder {
    ScoreBuilder::new()
  }

  #[must_use]
  pub const fn instructions(&self) -> &Instructions {
    &self.instructions
  }

  #[must_use]
  pub const fn criteria(&self) -> &ScoreCriteria {
    &self.criteria
  }
}

impl Question for ScoreQuestion {
  type Answer = ScoreAnswer;

  fn id(&self) -> QuestionId {
    self.id
  }

  fn extract(answer: &AnyAnswer) -> Option<&Self::Answer> {
    match answer {
      | AnyAnswer::Score(score) => Some(score),
      | _ => None,
    }
  }
}

#[derive(Clone, Debug, Default, Serialize)]
#[serde(transparent)]
pub struct ScoreCriteria(Vec<InputContent>);

impl ScoreCriteria {
  /// Levels in rubric order: the first item is level 0.
  pub fn iter(&self) -> impl Iterator<Item = &InputContent> {
    self
      .0
      .iter()
  }

  fn push(&mut self, level: InputContent) {
    self
      .0
      .push(level);
  }

  #[must_use]
  pub const fn len(&self) -> usize {
    self
      .0
      .len()
  }

  #[must_use]
  pub const fn is_empty(&self) -> bool {
    self
      .0
      .is_empty()
  }
}

#[derive(Debug)]
pub struct ScoreBuilder {
  instructions: Option<Instructions>,
  criteria:     ScoreCriteria,
}

impl ScoreBuilder {
  /// Entry point is `ScoreQuestion::builder()`.
  fn new() -> Self {
    Self {
      instructions: None,
      criteria:     ScoreCriteria::default(),
    }
  }

  #[must_use]
  pub fn instructions(mut self, instructions: impl Into<Instructions>) -> Self {
    self.instructions = Some(instructions.into());
    self
  }

  #[must_use]
  /// Adds the next rubric level. Call order sets the level number: the first
  /// call is level 0 (lowest), the next is level 1, and so on.
  pub fn level(mut self, level: impl Into<InputContent>) -> Self {
    self
      .criteria
      .push(level.into());
    self
  }

  pub fn build(self) -> Result<ScoreQuestion, JevvyError> {
    let instructions = self
      .instructions
      .ok_or(JevvyError::MissingInstructions)?;

    let count = self
      .criteria
      .len();

    if !(2..=10).contains(&count) {
      return Err(JevvyError::InvalidScoreLevelCount { count });
    }

    Ok(ScoreQuestion {
      id: QuestionId::generate(),
      instructions,
      criteria: self.criteria,
    })
  }
}

/// Jev's answer to a `ScoreQuestion`.
#[derive(Debug, Deserialize)]
pub struct ScoreAnswer {
  /// The probability-weighted answer across the levels; can land between levels.
  score:         f64,
  /// Each level number mapped back to its description.
  legend:        ScoreLegend,
  /// Each level (string key) mapped to its probability (floats that sum to 1).
  probabilities: Probabilities,
  /// How certain the model is, derived from probabilities.
  confidence:    Confidence,
}

impl ScoreAnswer {
  /// The probability-weighted answer across the levels; can land between levels.
  #[must_use]
  pub const fn score(&self) -> f64 {
    self.score
  }

  /// Each level number mapped back to its description.
  #[must_use]
  pub const fn legend(&self) -> &ScoreLegend {
    &self.legend
  }

  /// Each level (string key) mapped to its probability (floats that sum to 1).
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

#[derive(Debug, Deserialize)]
#[serde(transparent)]
pub struct ScoreLegend(HashMap<String, String>);

impl ScoreLegend {
  pub fn iter(&self) -> impl Iterator<Item = (&str, &str)> {
    self
      .0
      .iter()
      .map(|(level, description)| (level.as_str(), description.as_str()))
  }
}

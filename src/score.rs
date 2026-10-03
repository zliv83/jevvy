use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{
  error::JevvyError,
  questions::Question,
  reply::AnyAnswer,
  types::{Confidence, InputContent, Instructions, Probabilities, QuestionId},
};

#[derive(Clone, Debug, Serialize)]
/// A question that asks Jev to rate the state using levels you define.
/// Jev weights each level number by its probability to calculate the score.
///
/// Build one with `ScoreQuestion::builder()`.
pub struct ScoreQuestion {
  #[serde(skip)]
  id:           QuestionId,
  /// What the model should rate. Use an object to include the question
  /// in one field and supporting data in others.
  instructions: Instructions,
  /// An array of level descriptions, ordered from lowest to highest.
  /// The API requires 2 to 10 levels.
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
  /// Yields the levels in rubric order, starting at level 0.
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
  /// Creates an empty builder for `ScoreQuestion::builder()`.
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
  /// Adds a level to the rubric, from lowest to highest.
  /// The first level you add is 0, the next is 1, and so on.
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
  /// The average level number, weighted by probability.
  /// The score can fall between levels.
  score:         f64,
  /// Level descriptions, keyed by level number.
  legend:        ScoreLegend,
  /// The probability of each level, keyed by level number as a string.
  /// The values sum to 1.
  probabilities: Probabilities,
  /// The model's certainty, calculated from the probabilities.
  confidence:    Confidence,
}

impl ScoreAnswer {
  /// The average level number, weighted by probability.
  /// The score can fall between levels.
  #[must_use]
  pub const fn score(&self) -> f64 {
    self.score
  }

  /// Level descriptions, keyed by level number.
  #[must_use]
  pub const fn legend(&self) -> &ScoreLegend {
    &self.legend
  }

  /// The probability of each level, keyed by level number as a string.
  /// The values sum to 1.
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

use crate::{
	traits::{Levels, Options},
	types::{
		answers::{ChoiceAnswer, ScoreAnswer},
		error::JevvyError,
	},
};
use indexmap::IndexMap;

/// A Choice answer, turned into your own enum
#[derive(Debug, Clone)]
pub struct Choice<T> {
  /// The option the model picked.
  pub value: T,

  /// How sure it is, from 0 to 1.
  pub confidence: f64,

  /// Every option's probability, in the order you listed them.
  pub probabilities: IndexMap<T, f64>,
}

impl<T> Choice<T> {
  /// The picked option, but only if confidence is at least `min`.
  pub fn confident(&self, min: f64) -> Option<&T> {
    (self.confidence >= min).then_some(&self.value)
  }
}

impl<T: Options> TryFrom<&ChoiceAnswer> for Choice<T> {
  type Error = JevvyError;

  fn try_from(answer: &ChoiceAnswer) -> Result<Self, Self::Error> {
    Ok(Choice {
      value:         lookup(&answer.choice)?,
      confidence:    answer.confidence,
      probabilities: answer
        .probabilities
        .iter()
        // Each name becomes Ok((variant, p)) or Err(UnknownOption).
        .map(|(name, p)| lookup(name).map(|option| (option, *p)))
        // Many Results become one Result. It stops at the first Err.
        .collect::<Result<_, _>>()?,
    })
  }
}

fn lookup<T: Options>(name: &str) -> Result<T, JevvyError> {
  T::from_name(name).ok_or_else(|| JevvyError::UnknownOption(name.to_owned()))
}

/// A score answer, matched to your own enum of levels.
#[derive(Debug, Clone)]
pub struct Score<T> {
  /// The nearest level, e.g. `Severity::Annoying`
  pub level:         T,
  /// The exact spot on the scale
  pub raw:           f64,
  pub confidence:    f64,
  pub probabilities: IndexMap<T, f64>,
}

impl<T: Levels> TryFrom<&ScoreAnswer> for Score<T> {
  type Error = JevvyError;

  fn try_from(answer: &ScoreAnswer) -> Result<Self, Self::Error> {
    Ok(Score {
      level:         T::nearest(answer.score),
      raw:           answer.score,
      confidence:    answer.confidence,
      probabilities: answer
        .probabilities
        .iter()
        .map(|(key, p)| level_at(key).map(|level| (level, *p)))
        .collect::<Result<_, _>>()?,
    })
  }
}

/// Turns a positoin key from the API ("1") into a level, or an error.
fn level_at<T: Levels>(key: &str) -> Result<T, JevvyError> {
  key
    .parse()
    .ok()
    .and_then(T::from_index)
    .ok_or_else(|| JevvyError::UnknownLevel(key.to_owned()))
}

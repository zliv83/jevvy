#![allow(clippy::cast_precision_loss)]
use crate::{
  traits::{Levels, Options},
  types::{
    answers::{ChoiceAnswer, Confidence, Probabilities, ScoreAnswer},
    jevvy_error::JevvyError,
  },
};

/// A Choice answer, read into your own enum.
#[derive(Debug, Clone)]
pub struct Choice<T> {
  /// The option the model picked (the most likely one).
  pub choice:        T,
  /// Every option's probability, in the order the API sent them.
  pub probabilities: Probabilities<T>,
  /// How sure the model is, from 0 to 1.
  pub confidence:    f64,
}

impl<T> Choice<T> {
  /// The pick, but only if confidence is at least `min`.
  #[must_use]
  pub fn confidence(&self, min: f64) -> Option<&T> {
    (self.confidence >= min).then_some(&self.choice)
  }
}

impl<T: Options> Choice<T> {
  /// Reads the API's Choice answer under `key` into your enum.
  ///
  /// `key` is only for the error, so it can say which question named
  /// an option your enum doesn't have.
  ///
  /// # Errors
  ///
  /// [`JevvyError::UnknownOption`] if the API named an option that isn't
  /// in `T::ALL`.
  pub fn from_answer(key: &str, answer: &ChoiceAnswer) -> Result<Self, JevvyError> {
    Ok(Choice {
      choice:        lookup(key, &answer.choice)?,
      probabilities: answer
        .probabilities
        .iter()
        .map(|(name, p)| lookup(key, name).map(|option| (option, *p)))
        .collect::<Result<_, _>>()?,
      confidence:    answer.confidence,
    })
  }
}

/// Turns an option name from the API back into a variant, or an error
/// that names both question and the stray option.
fn lookup<T: Options>(key: &str, name: &str) -> Result<T, JevvyError> {
  T::from_name(name).ok_or_else(|| JevvyError::UnknownOption {
    key:   key.to_owned(),
    found: name.to_owned(),
  })
}

/// A score answer, read as a position on your scale.
///
/// `score` is the headline. A weighted spot on the line from level 0
/// to the top level, and it can land between levels. Rounding to a level
/// is something you can ask for with [`Score::nearest`] or [`Score::most_likely`]
#[derive(Debug, Clone)]
pub struct Score<T> {
  /// Reads API's Socre answer under `key` into your levels.
  pub score:         f64,
  /// Every level's probability, in the order the API sent them.
  pub probabilities: Probabilities<T>,
  /// HOw sure the model is, from 0 to 1.
  pub confidence:    Confidence,
}

impl<T: Levels> Score<T> {
  /// Reads the API's Score answer under `key` into your levels.
  ///
  /// # Errors
  ///
  /// [`JevvyError::UnknownLevel`] if the API sent a position your
  /// enum has no level for.
  pub fn from_answer(key: &str, answer: &ScoreAnswer) -> Result<Self, JevvyError> {
    Ok(Score {
      score:         answer.score,
      probabilities: answer
        .probabilities
        .iter()
        .map(|(position, p)| level_at(key, position).map(|level| (level, *p)))
        .collect::<Result<_, _>>()?,
      confidence:    answer.confidence,
    })
  }

  /// The level closest to `score`: 1.05 rounds to level 1.
  ///
  /// Careful with a split: 50% bottom and 50% top averages to the
  /// middle, a level the model never picked. Check `confidence`, or
  /// use [`Score::most_likely`].
  ///
  /// # Panics
  ///
  /// If `T::ALL` is empty. The API needes at least 2 levels anyway.
  #[must_use]
  pub fn nearest(&self) -> T {
    let last = T::ALL.len() - 1;
    let index = self
      .score
      .round()
      .clamp(0.0, last as f64) as usize;
    T::ALL[index]
  }

  /// The single level with the highest probability. Ties go
  /// to the higher level.
  #[must_use]
  pub fn most_likely(&self) -> T {
    self
      .probabilities
      .iter()
      // total_comp orders f64's savely, even odd ones like NaN.
      .max_by(|a, b| {
        a.1
          .total_cmp(b.1)
      })
      .map(|(level, _)| *level)
      // No probabilities at all shouldn't happen. Fall back to rounding.
      .unwrap_or_else(|| self.nearest())
  }

  /// `score` squeezed onto 0..=1, so scales off different sizes
  /// compare. 1.0 on a 3-level scale (0..=2) is 0.5.
  #[must_use]
  pub fn normalized(&self) -> f64 {
    let top = (T::ALL.len() - 1) as f64;
    self.score / top
  }

  /// True when `score` has reached `level`'s position or beyond.
  ///
  /// `severity.at_least(Severity::Annoying)` is true for 1.0 and 1.7.
  #[must_use]
  pub fn at_least(&self, level: T) -> bool {
    self.score >= level.index() as f64
  }
}

/// Turns a position key from the API (`"1"`) into a level, or an
/// error that names both the question and the stray position.
fn level_at<T: Levels>(key: &str, position: &str) -> Result<T, JevvyError> {
  position
    .parse()
    .ok()
    .and_then(T::from_index)
    .ok_or_else(|| JevvyError::UnknownLevel {
      key:   key.to_owned(),
      found: position.to_owned(),
    })
}

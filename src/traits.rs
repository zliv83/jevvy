use crate::types::{
  jevvy_error::JevvyError,
  jevvy_response::JevvyResponse,
  questions::{ChoiceOption, Question, Questions},
  Entry,
};
use std::hash::Hash;

/// An enum whose variants are the options of a Choice question.
///
/// `Copy + Eq + Hash` lets a variant be a key in `Choice::probabilities`.
pub trait Options: Copy + Eq + Hash + 'static {
  /// Every variant, in the order the model should see them.
  const ALL: &'static [Self];

  /// The name sent to the API, e.g. `"networking"`.
  fn name(&self) -> &'static str;

  /// What this option means. `None` sends no description.
  fn describe(&self) -> Option<&'static str> {
    None
  }

  /// Turns a name from the API back into a variant.
  fn from_name(name: &str) -> Option<Self> {
    Self::ALL
      .iter()
      .copied()
      .find(|option| option.name() == name)
  }

  /// A Choice question that offers every variant.
  fn question(instructions: impl Into<Entry>) -> Question {
    Question::choice(
      instructions,
      Self::ALL
        .iter()
        .map(|option| ChoiceOption {
          name:        option
            .name()
            .into(),
          description: option
            .describe()
            .map(Entry::from),
        }),
    )
  }
}

/// An enum whose variants are the levels of a Score question,
/// from lowest to highest.
pub trait Levels: Copy + Eq + Hash + 'static {
  /// Every level, lowest first. This order *is* the scale.
  const ALL: &'static [Self];

  /// The text the model reads for this level.
  fn describe(&self) -> &'static str;

  /// The level at position `index` (0 is the lowest).
  fn from_index(index: usize) -> Option<Self> {
    Self::ALL
      .get(index)
      .copied()
  }

  /// The level closest to a raw score, e.g. 1.05 -> position 1.
  ///
  /// # Panics
  ///
  /// If `ALL` is empty. The API requires at least 2 levels anyway.
  fn nearest(raw: f64) -> Self {
    let last = Self::ALL.len() - 1;
    // clamp keeps od sources (like -1.0 or 2.7) on the scale
    let index = raw
      .round()
      .clamp(0.0, last as f64) as usize;
    Self::ALL[index]
  }

  /// A Score question with every level, lowest first.
  fn question(instructions: impl Into<Entry>) -> Question {
    Question::score(
      instructions,
      Self::ALL
        .iter()
        .map(|level| level.describe()),
    )
  }
}

/// A struct whose fields are the answers to a set of questions.
///
/// One field = one question. The field name is the question's key.
pub trait Rubric: Sized {
  /// Every question to ask, keyed by field name.
  fn questions() -> Questions;

  /// Fills the struct from the API's answers.
  ///
  /// # Errors
  ///
  /// If an answer is missing, is the wrong kind, or names
  /// an option or level the field's enum doesn't have.
  fn from_response(response: &JevvyResponse) -> Result<Self, JevvyError>;
}

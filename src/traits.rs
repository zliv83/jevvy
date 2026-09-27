use std::hash::Hash;

#[cfg(feature = "derive")]
pub use jevvy_derive::Options;

use crate::{
  sheet::{AnswerSheet, QuestionSheet},
  types::{
    jevvy_error::JevvyError,
    questions::{ChoiceOption, Question},
    Entry,
  },
};

/// An enum whose variants are the options of a Choice question.
///
/// `Copy + Eq + Hash` lets a variant be a key in `Choice::probabilities`.
pub trait Options: Copy + Eq + Hash + 'static {
  /// Every variant, in order the model should see them.
  const ALL: &'static [Self];

  /// The name sent to the API, e.g. `"networking"`.
  fn name(&self) -> &'static str;

  /// What this option means, as text or a structured object.
  /// `None` sends `null`.
  fn describe(&self) -> Option<Entry> {
    None
  }

  /// Turns a name from the API back into a variant.
  fn from_name(name: &str) -> Option<Self> {
    Self::ALL
      .iter()
      .copied()
      .find(|option| option.name() == name)
  }

  /// A Choice question that offers every variant, in `ALL` order.
  fn question(instructions: impl Into<Entry>) -> Question {
    Question::choice(
      instructions,
      Self::ALL
        .iter()
        .map(|option| ChoiceOption {
          name:        option
            .name()
            .into(),
          description: option.describe(),
        }),
    )
  }
}

/// An enum whose variants are the levels fo a Score question,
/// from lowest to highest.
pub trait Levels: Copy + Eq + Hash + 'static {
  /// Every level, lowest first. This order *is* the scale.
  const ALL: &'static [Self];

  /// What this level means, as text or a structured object.
  fn describe(&self) -> Entry;

  /// The level at postition `index` (0 is the lowest).
  fn from_index(index: usize) -> Option<Self> {
    Self::ALL
      .get(index)
      .copied()
  }

  /// This level's position on the scale. 0 for the first in `ALL`.
  ///
  /// # Panics
  ///
  /// If `self` isn't in `ALL`, which means `ALL` is missing a variant.
  fn index(&self) -> usize {
    Self::ALL
      .iter()
      .position(|level| level == self)
      .expect("every level must be listed in Levels::ALL")
  }

  /// A Score question with every level, lowest first.
  fn question(instructions: impl Into<Entry>) -> Question {
    // `Self::describe` is the method used as a fucntion: &Self -> Entry.
    Question::score(
      instructions,
      Self::ALL
        .iter()
        .map(Self::describe),
    )
  }
}

/// A set of questions with a typed answer sheet on the back.
///
/// A form *contributes* questions to a request it doesn't own.
/// `ask` writes onto the sheet it's handed, and `read` reads back
/// from the matching sheet with the same short keys.
///
/// A unit struct(`struct TriageForm;`) is a constant form.
/// A struct with fields (`struct SamePerson<'a> { record: &'a Candidate })`
/// is a form build from data. Same trait for both.
pub trait Form {
  /// What `read` fiills in, e.g. `Triage { team, frustration, ..} `.
  type Answers;

  /// Writes the form's questions onto `sheet`.
  fn ask(&self, sheet: &mut QuestionSheet<'_>);

  /// Reads this form's answers back from `sheet`.
  ///
  /// # Errors
  ///
  /// A missing answer, the wrong kind, or an option or level your
  /// enum doesn't have.
  fn read(&self, sheet: &AnswerSheet<'_>) -> Result<Self::Answers, JevvyError>;
}

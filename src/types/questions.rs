use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

use crate::types::{jevvy_error::JevvyError, Entries, Entry};

/// The most options one Choice can offer.
pub const MAX_OPTIONS: usize = 255;

/// The fewest levels one Score can have. One level isn't a scale.
pub const MIN_LEVELS: usize = 2;

/// The most levels one Score can have.
pub const MAX_LEVELS: usize = 10;

/// Every question in a request, keyed by the id you choose.
///
/// Order is kept, so the request JSON reads in the order you wrote it.
pub type Questions = IndexMap<String, Question>;

/// One judgement about the state. The `type` field designates which question.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Question {
  /// Ys or no. Th answer is the probability of yes, from 0 to 1.
  Noul {
    /// the yes/no question.
    instructions: Entry,
    /// What a yes and no mean. Skips if 'None`.
    #[serde(skip_serializing_if = "Option::is_none")]
    criteria:     Option<NoulCriteria>,
  },

  /// Pick one option from a set you wrote.
  Choice {
    /// What the model should decide.
    instructions: Entry,
    /// Option name to descrtiption. `None` sends `null`.
    criteria:     IndexMap<String, Option<Entry>>,
  },

  /// A postion on an ordered scale, lowest level first.
  Score {
    /// What the model should rate.
    instructions: Entry,
    /// The levels, lowest first. The order *is* the scale.
    criteria:     Entries,
  },
}

impl Question {
  /// A yes/no question. The answer is the probability of "yes".
  pub fn noul(instructions: impl Into<Entry>) -> Self {
    Question::Noul {
      instructions: instructions.into(),
      criteria:     None,
    }
  }

  /// Rates the state on an ordered scale, from lowest to highest.
  pub fn score(
    instructions: impl Into<Entry>,
    levels: impl IntoIterator<Item = impl Into<Entry>>,
  ) -> Self {
    Question::Score {
      instructions: instructions.into(),
      criteria:     levels
        .into_iter()
        .map(Into::into)
        .collect(),
    }
  }

  /// Picks one option from a set.
  pub fn choice(
    instructions: impl Into<Entry>,
    options: impl IntoIterator<Item = impl Into<ChoiceOption>>,
  ) -> Self {
    Question::Choice {
      instructions: instructions.into(),
      criteria:     options
        .into_iter()
        .map(|option| {
          let option = option.into();
          (option.name, option.description)
        })
        .collect(),
    }
  }

  /// Checks the API's limits before anything leaves the machine.
  ///
  /// A choice needs 1 to 255 options, and a Score needs 2 to 10 levels.
  /// A Noul has nothing to count, so it always passes.
  ///
  /// `key` is the question's id in the request. A question doesn't know its
  /// own id, so it's passed in to say which question broke the rule.
  ///
  /// # Errors
  ///
  /// [`JevvyError::BadOptionCount`] or [`JevvyError::BadLevelCount`].
  pub fn validate(&self, key: &str) -> Result<(), JevvyError> {
    match self {
      | Question::Noul { .. } => Ok(()),
      | Question::Choice { criteria, .. } => {
        let count = criteria.len();
        if (1..=MAX_OPTIONS).contains(&count) {
          Ok(())
        } else {
          Err(JevvyError::BadOptionCount {
            key: key.to_owned(),
            count,
          })
        }
      }
      | Question::Score { criteria, .. } => {
        let count = criteria.len();
        if (MIN_LEVELS..=MAX_LEVELS).contains(&count) {
          Ok(())
        } else {
          Err(JevvyError::BadLevelCount {
            key: key.to_owned(),
            count,
          })
        }
      }
    }
  }
}

/// What a yes and a no mean for a Noul. Either side can be left off.
///
/// The docs call the two sides `true` and `false`. Those are rust keywords,
/// so the fields are `when_true` and `when_false`, and serde renames them.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoulCriteria {
  /// What a yes (near 1) means.
  #[serde(rename = "true", skip_serializing_if = "Option::is_none")]
  pub when_true:  Option<Entry>,
  #[serde(rename = "false", skip_serializing_if = "Option::is_none")]
  pub when_false: Option<Entry>,
}

/// One option in a Choice question - a name, plus an optional description
#[derive(Debug, Clone)]
pub struct ChoiceOption {
  /// The option's name, sent as a key in `criteria`, e.g. `"billing"`.
  pub name:        String,
  /// What the option means. `None` sends `null`.
  pub description: Option<Entry>,
}

/// Just a name, i.e. "sales"
impl From<&str> for ChoiceOption {
  fn from(name: &str) -> Self {
    Self {
      name:        name.into(),
      description: None,
    }
  }
}

/// So names from a Vec<String> work too.
impl From<String> for ChoiceOption {
  fn from(name: String) -> Self {
    Self {
      name,
      description: None,
    }
  }
}

/// A name and a description: ("billing", "Payments, refunds")
impl<N: Into<String>, D: Into<Entry>> From<(N, D)> for ChoiceOption {
  fn from((name, description): (N, D)) -> Self {
    Self {
      name:        name.into(),
      description: Some(description.into()),
    }
  }
}

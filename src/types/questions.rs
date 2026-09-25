use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

use crate::types::{Entries, Entry};
pub type Questions = IndexMap<String, Question>;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Question {
  Noul {
    #[serde(skip_serializing_if = "Option::is_none")]
    instructions: Option<Entry>,
    #[serde(skip_serializing_if = "Option::is_none")]
    criteria:     Option<NoulCriteria>,
  },
  Choice {
    /// The question the model answers
    #[serde(skip_serializing_if = "Option::is_none")]
    instructions: Option<Entry>,
    /// The answer options, as a map. Each key is an option name
    /// and each value is a description of that option.
    criteria:     IndexMap<String, Option<Entry>>,
  },
  Score {
    #[serde(skip_serializing_if = "Option::is_none")]
    instructions: Option<Entry>,
    criteria:     Entries,
  },
}

impl Question {
  /// A yes/no question. The answer is the probability of "yes".
  pub fn noul(instructions: impl Into<Entry>) -> Self {
    Question::Noul {
      instructions: Some(instructions.into()),
      criteria:     None,
    }
  }

  /// Rates the state on an ordered scale, from lowest to highest.
  pub fn score(
    instructions: impl Into<Entry>,
    levels: impl IntoIterator<Item = impl Into<Entry>>,
  ) -> Self {
    Question::Score {
      instructions: Some(instructions.into()),
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
      instructions: Some(instructions.into()),
      criteria:     options
        .into_iter()
        .map(|option| {
          let option = option.into();
          (option.name, option.description)
        })
        .collect(),
    }
  }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoulCriteria {
  #[serde(rename = "true")]
  pub yes: Option<Entry>,
  #[serde(rename = "false")]
  pub no:  Option<Entry>,
}

/// One option in a Choice question - a name, plus an optional description
pub struct ChoiceOption {
  pub name:        String,
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

/// A name and a description: ("billing", "Payments, refunds")
impl<N: Into<String>, D: Into<Entry>> From<(N, D)> for ChoiceOption {
  fn from((name, description): (N, D)) -> Self {
    Self {
      name:        name.into(),
      description: Some(description.into()),
    }
  }
}

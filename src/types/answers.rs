use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

type Probabilities = IndexMap<String, f64>;
type Confidence = f64;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Answer {
  Noul(NoulAnswer),
  Choice(ChoiceAnswer),
  Score(ScoreAnswer),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoulAnswer {
  /// Probability of "yes", from 0 to 1.
  pub noul: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChoiceAnswer {
  /// The most likely option
  pub choice:        String,
  pub probabilities: Probabilities,
  pub confidence:    Confidence,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoreAnswer {
  /// Weighted position on the scale. It can land between levels.
  pub score:         f64,
  pub legend:        IndexMap<String, String>,
  pub probabilities: Probabilities,
  pub confidence:    Confidence,
}

impl Answer {
  /// The kind of answer as a word. Used in error messages.
  pub fn kind(&self) -> &'static str {
    match self {
      | Answer::Noul(_) => "noul",
      | Answer::Choice(_) => "choice",
      | Answer::Score(_) => "score",
    }
  }
}

pub type Answers = IndexMap<String, Answer>;

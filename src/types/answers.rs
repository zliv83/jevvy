use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

type Probabilities = IndexMap<String, f64>;
type Confidence = f64;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Answer {
  Noul {
    noul: f64,
  },
  Choice {
    choice:        String,
    probabilities: Probabilities,
    confidence:    Confidence,
  },
  Score {
    score:         f64,
    legend:        IndexMap<String, String>,
    probabilities: Probabilities,
    confidence:    Confidence,
  },
}

pub type Answers = IndexMap<String, Answer>;

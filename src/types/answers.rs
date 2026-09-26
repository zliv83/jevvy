use crate::types::Entry;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

/// The chance of each thing, in order. The values sum to 1.
///
/// Keyed by `String` on the wire (`"billing"`, `"0"`), and by your own
/// enum once typed (`Probabilities<Team>`). `K = String` is a default:
/// say nothing and you get a `String`.
pub type Probabilities<K = String> = IndexMap<K, f64>;

/// Level number (`"0"`, `"1"`, ...) to the description you sent.
pub type Legend = IndexMap<String, Entry>;

/// How sure the model is, from 0 to 1, worked out from the probabilities.
type Confidence = f64;

/// Option name to description, in the order the model reads them.
/// `None` sends `null`: the name is enough
pub type ChoiceCriteria = IndexMap<String, Option<Entry>>;

/// One answer, returned under the same id as its question.
///
/// The `type` field says which kind it is.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Answer {
  /// The answer to a true/false question.
  Noul(NoulAnswer),
  /// The answer to a pick-one question.
  Choice(ChoiceAnswer),
  /// The answer to a rate-on-a-scale question.
  Score(ScoreAnswer),
}

/// The answer to a Noul.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoulAnswer {
  /// Probability of true, from 0 (false) to 1 (true).
  pub noul: f64,
}

/// The answer to a Choice.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChoiceAnswer {
  /// The most likely option's name.
  pub choice:        String,
  /// Every option's probability, keyed by name.
  pub probabilities: Probabilities,
  /// How sure the model is, from 0 to 1.
  pub confidence:    Confidence,
}

/// The answer to a Score.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoreAnswer {
  /// The probability-weighted position on the scale. It can land between
  /// levels: `1.05` is just past level 1.
  pub score:         f64,
  /// Each level number (`"0"`, `"1"`, ...) mapped back t the description
  /// you send. An `Entry`, because a level can be an object, not just text.
  pub legend:        Legend,
  /// Each level number's probability.
  pub probabilities: Probabilities,
  /// How sure the model is, from 0 to 1.
  pub confidence:    Confidence,
}

impl Answer {
  /// The kind of answer as a word. Used in error messages.
  #[must_use]
  pub fn kind(&self) -> &'static str {
    match self {
      | Answer::Noul(_) => "noul",
      | Answer::Choice(_) => "choice",
      | Answer::Score(_) => "score",
    }
  }
}

/// Every answer in a response, keyed by a question id.
///
/// A thin wrapper around the map. `#[serde(transparent)]` means it reads
/// and writes as the plain map with no extra layer.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Answers(IndexMap<String, Answer>);

impl Answers {
  /// The answer under `key`, if there is one.
  #[must_use]
  pub fn get(&self, key: &str) -> Option<&Answer> {
    self
      .0
      .get(key)
  }

  /// Every `(key, answer)` pair, in the order the API sent them.
  pub fn iter(&self) -> impl Iterator<Item = (&str, &Answer)> {
    self
      .0
      .iter()
      // Hand out `&str` instead of `&String`, which is easier to use.
      .map(|(key, answer)| (key.as_str(), answer))
  }

  /// How many answers there are.
  #[must_use]
  pub fn len(&self) -> usize {
    self
      .0
      .len()
  }

  /// True when there are no answers at all.
  #[must_use]
  pub fn is_empty(&self) -> bool {
    self
      .0
      .is_empty()
  }
}

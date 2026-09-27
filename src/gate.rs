//! Gates: turning an answer and its confidence into act, confirm, or escalate.
//!
//! Jev says how sure it is. A gate is your rule for what "sure enough" means
//! for one action. Theasholds live here, in your code, named for the action
//! they guard.

use crate::types::{
  answers::Confidence,
  typed::{Choice, Score},
};

/// What to do with an answer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict<T> {
  /// Sure enough to go ahead on its own.
  Act(T),
  /// Probably right to go ahead, but check with a hooman first.🧍🧍‍♀️
  Confirm(T),
  /// Not sure: hand it to a hooman. 🤷‍♂️
  Escalate,
}

impl<T> Verdict<T> {
  /// True only for [`Verdict::Act`].
  #[must_use]
  pub fn is_sure(&self) -> bool {
    matches!(self, Verdict::Act(_))
  }

  /// Changes the value inside, keeping the verdict.
  /// `Act(Team::Billing).gate_map(|t| t.name())` is `Act("billing").
  pub fn gate_map<U>(self, f: impl FnOnce(T) -> U) -> Verdict<U> {
    match self {
      | Verdict::Act(value) => Verdict::Act(f(value)),
      | Verdict::Confirm(value) => Verdict::Confirm(f(value)),
      | Verdict::Escalate => Verdict::Escalate,
    }
  }

  /// The value for 'Act` of `Confirm`, `None` for `Escalate`.
  #[must_use]
  pub fn into_option(self) -> Option<T> {
    match self {
      | Verdict::Act(value) | Verdict::Confirm(value) => Some(value),
      | Verdict::Escalate => None,
    }
  }
}

/// Confidence thresholds for a Choice or Score.
///
///
/// ``` text
///  0 ───────────── confirm ───────────── act ───────────── 1
///      Escalate            Confirm              Act
/// ```
///
/// Set `confirm` equal to `act` for no confirm band at all. Name each
/// gate for the action it guards:
/// `const APPROVE_TRANSFER: Gate = Gate { act: 0.85, confirm: 0.6 };`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Gate {
  /// At or above this confidence, act.
  pub act:     f64,
  /// At or above this (but below `act`), confirm first.
  pub confirm: f64,
}

impl Gate {
  /// Sorts a confidence into a verdict, carrying `value` along.
  fn sort<T>(&self, confidence: Confidence, value: T) -> Verdict<T> {
    if confidence >= self.act {
      Verdict::Act(value)
    } else if confidence >= self.confirm {
      Verdict::Confirm(value)
    } else {
      Verdict::Escalate
    }
  }

  /// Judges a Choice by its confidence. The verdict carries the pick.
  #[must_use]
  pub fn judge<T: Copy>(&self, choice: &Choice<T>) -> Verdict<T> {
    self.sort(choice.confidence, choice.choice)
  }

  /// Judges a Score by its confidence. The verdict carries the position.
  #[must_use]
  pub fn judge_score<T>(&self, score: &Score<T>) -> Verdict<f64> {
    self.sort(score.confidence, score.score)
  }
}

/// Thresholds for a Noul, which has no confidence of its own.
/// The probability itself says how sure it is.
///
/// ```text
///  0 ────── no ───────────────── yes ────── 1
///   Act(false)       Escalate        Act(true)
/// ```
/// TypeSafe.ai's usual spit is `NoulGate { yes: 0.8, no: 0.2 }`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NoulGate {
  /// At or above this, the answer is yes.
  pub yes: f64,
  /// At or below this, the answer is no.
  pub no:  f64,
}

impl NoulGate {
  /// Judges the probability of yes. The band between `no` and `yes` escalates.
  #[must_use]
  pub fn judge(&self, noul: f64) -> Verdict<bool> {
    if noul >= self.yes {
      Verdict::Act(true)
    } else if noul <= self.no {
      Verdict::Act(false)
    } else {
      Verdict::Escalate
    }
  }
}

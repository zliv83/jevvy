use serde::{Deserialize, Serialize};

use crate::types::{questions::Questions, Entry};
/// Request structure
///
/// The POST request body to the TypeSafe.ai API has a specific structure.
/// The top level has three fields:
///    1. state - the content to evaluate
///    2. model - the model used to evaluate
///    3. questions - a map from question ids you choose to question objects
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Request {
  /// State - the content to eval
  pub state: Entry,

  /// Model - the model used to eval
  pub model: String,

  /// Questions - a map of questions
  pub questions: Questions,
}

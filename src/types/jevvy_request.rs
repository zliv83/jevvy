use serde::{Deserialize, Serialize};

use crate::types::{questions::Questions, Entry};
/// One requst. The body POSTed to `/v1/systemone`.
///
/// One state, one model, and every question about this moemnt.
/// Named 'JevvyRequest` so it's never mistaken for `reqwest::Request`.
///
/// The top level has three fields:
///    1. state - the content to evaluate
///    2. model - the model used to evaluate
///    3. questions - a map from question ids you choose to question objects
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JevvyRequest {
  /// State - the content to eval
  pub state: Entry,

  /// Model - the model used to eval
  pub model: String,

  /// Questions - a map of questions
  pub questions: Questions,
}

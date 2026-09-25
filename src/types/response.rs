use serde::{Deserialize, Serialize};

use crate::types::answers::Answers;

/// One answer per question, returned under the same ids you provided

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Response {
  /// Model - The model that performed the eval
  pub model: String,

  /// The answers
  pub answers: Answers,

  /// Token usage for the request
  pub usage: Usage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Usage {
  pub input_tokens:  f64,
  pub output_tokens: f64,
}

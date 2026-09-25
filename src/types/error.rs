use reqwest::{Response, StatusCode};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiErrorPayload {
  pub message: Option<String>,
  pub field:   Option<String>,
  pub detail:  Option<String>,
}

#[derive(Debug, Error)]
pub enum JevvyError {
  /// `TYPESAFE_API_KEY` isn't set
  #[error("TYPESAFE_API_KEY environment variable is not set")]
  MissingApiKey,

  // Underlying network, DNS, or serialization failure from reqwest
  #[error("HTTP request error: {0}")]
  Request(#[from] reqwest::Error),

  #[error("Unauthorized (401): check your API key")]
  Unauthorized,

  /// Missing or invalidated API Key
  #[error("Unauthorized (422) on field `{field:?}: {message}")]
  UnprocessableEntity {
    field:   Option<String>,
    message: String,
  },

  /// Rate limit hit
  #[error("Rate limit exceeded (429): back off and retry")]
  RateLimited,

  /// TypeSafe.ai is temporarily overloaded
  #[error("Server overloaded (529): retry after a short delay")]
  Overloaded,

  /// Fallback for unexpected HTTP status codes
  #[error("API error ({status}): {message}")]
  Api {
    status:  StatusCode,
    message: String,
  },

  /// No answer came back under this key.
  #[error("no answer for `{0}`")]
  MissingAnswer(String),

  /// The answer exists, but it's a different kind.
  #[error("answer `{key}` is a {found}, not a {expected}")]
  WrongAnswerType {
    key:      String,
    expected: &'static str,
    found:    &'static str,
  },
}

/// Handles response for errors
///
/// # Errors
///
/// `JevvyError`
pub async fn handle_response(response: Response) -> Result<Response, JevvyError> {
  let status = response.status();

  if status.is_success() {
    return Ok(response);
  }

  // Attempt to extract the JSON error message - fallback to raw text if parsing fails
  let payload = response
    .json::<ApiErrorPayload>()
    .await
    .ok();
  let message = payload
    .as_ref()
    .and_then(|p| {
      p.message
        .clone()
        .or_else(|| {
          p.detail
            .clone()
        })
    })
    .unwrap_or_else(|| "Unknown API error".to_string());

  match status.as_u16() {
    | 401 => Err(JevvyError::Unauthorized),
    | 422 => Err(JevvyError::UnprocessableEntity {
      field: payload.and_then(|p| p.field),
      message,
    }),
    | 429 => Err(JevvyError::RateLimited),
    | 529 => Err(JevvyError::Overloaded),
    | _ => Err(JevvyError::Api { status, message }),
  }
}

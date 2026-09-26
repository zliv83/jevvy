use reqwest::{Response, StatusCode};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use thiserror::Error;

use crate::types::questions::{MAX_LEVELS, MAX_OPTIONS, MIN_LEVELS};

/// The JSON body the API sends back with an error.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiErrorPayload {
  /// What went wrong, in words.
  pub message: Option<String>,
  /// The request field that failed validation (on a 422).
  pub field:   Option<String>,
  /// Extra detail, used when `message` in missing.
  pub detail:  Option<String>,
}

/// Everything that can go wrong in jevvy.
///
/// The families: the call didn't go through (network, status code,
/// timeout), the reqeust was refulsed before sending (option and level
/// counts), or the reply didn't fit what you asked for (missing, wrong kind
/// unknown option or level).
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum JevvyError {
  // -- Setup --
  /// `TYPESAFE_API_KEY` isn't set
  #[error("TYPESAFE_API_KEY environment variable is not set")]
  MissingApiKey,

  // -- The Call --
  /// Network, DNS, TLS, or body-decoding trouble from reqwest.
  #[error("HTTP request error: {0}")]
  Reqwest(#[from] reqwest::Error),

  /// No reply arrived within the client's timeout (30 seconds by default).
  /// `{after:?}` prints a Duration.
  #[error("no reply from TypeSafe.ai within {after:?}")]
  Timeout {
    /// How long we waited.
    after: Duration,
  },

  /// 401: the API key is missing or wrong.
  #[error("Unauthorized (401): check your API key")]
  Unauthorized,

  /// 422: the request body failed the API's validation.
  #[error("Unauthorized (422){} on field `{field:?}`: {message}", at_field(.field.as_deref()))]
  UnprocessableEntity {
    /// The field the API pointed at, if it named one.
    field:   Option<String>,
    /// The API's explanation.
    message: String,
  },

  /// 429: Rate limit hit, and still hit after every retry.
  #[error("Rate limit exceeded (429): back off and retry")]
  RateLimited,

  /// 529: TypeSafe.ai is overloaded, and still was after every retry.
  #[error("Server overloaded (529): retry after a short delay")]
  Overloaded,

  /// Any other status code.
  #[error("API error ({status}): {message}")]
  Api {
    /// The HTTP status code.
    status:  StatusCode,
    /// The API's explanation, or "Unknown API error".
    message: String,
  },

  // -- Refused before sending --
  /// A choice with no options, or more than the API allows.
  #[error("choice `{key}` needs 1 to {max} options, found {count}", max = MAX_OPTIONS)]
  BadOptionCount {
    /// The question's id in the request.
    key:   String,
    /// How many options it had.
    count: usize,
  },

  /// A Score with too few levels to be a scale, or more than the API allows.
  #[error(
    "score `{key}` needs {min} to {max} levels, found {count}",
    min = MIN_LEVELS,
    max = MAX_LEVELS,
  )]
  BadLevelCount {
    /// The question's id in the request.
    key:   String,
    /// How many levels it had.
    count: usize,
  },

  // -- Reading the reply --
  /// No answer came back under this key.
  #[error("no answer for `{0}`")]
  MissingAnswer(String),

  /// The answer exists, but it's a different kind.
  #[error("answer `{key}` is a {found}, not a {expected}")]
  WrongAnswerType {
    /// The questions's id.
    key:      String,
    /// The kind you asked to read.
    expected: &'static str,
    /// The kind that came back.
    found:    &'static str,
  },

  /// The API named an option your enum doesn't have.
  #[error("answer `{key}` picked `{found}`, which isn't in your enum")]
  UnknownOption {
    /// The qeustions id
    key:   String,
    /// The option name the API sent.
    found: String,
  },

  /// The API named a level position your enum doesn't have.
  #[error("answer `{key}` has a level at position `{found}`, but your enum has no level there")]
  UnknownLevel {
    /// The question's id
    key:   String,
    /// The position API send, e.g. `"3"`.
    found: String,
  },
}

/// The "at field" part of the 422 message, or nothing if the API named no field
fn at_field(field: Option<&str>) -> String {
  field
    .map(|f| format!(" at `{f}`"))
    .unwrap_or_default()
}

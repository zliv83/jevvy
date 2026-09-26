//!How a request reaches the API, and how the reply comes back.
//!
//! [`Transport`] is the socket. [`Http`] is the plug that goes to
//! TypeSafe.ai.

use std::time::Duration;

use reqwest::header::RETRY_AFTER;

use crate::types::{
  jevvy_error::{ApiErrorPayload, JevvyError},
  jevvy_request::JevvyRequest,
  jevvy_response::JevvyResponse,
};

/// Where requests go unless you say otherwise.
pub const DEFAULT_BASE_URL: &str = "https://api.typesafe.ai";

/// How many extra tries a busy API (429 or 529) gets.
pub const DEFAULT_MAX_TRIES: u32 = 5;

/// How long to wait for a reply before giving up.
pub const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);

/// The longes we sleep between retries, even if the API asks for more.
const MAX_WAIT: Duration = Duration::from_secs(60);

/// Anything that can take a [`Reqeust`] and hand back a [`Response`].
///
/// The client doesn't care how mthe reply is produced: over HTTPS, from
/// a file of saved receipts, or by a fake in a test. It only needs `send`.
pub trait Transport {
  /// Sends one request and retuns its reply.
  ///
  ///
  /// The `+ Send` means the waiting can be handed to another thread,
  /// so a client works inside `tokio::spawn`.
  ///
  /// # Errors
  ///
  /// Whatever went wrong on the way: see [`JevvyError`].
  fn send(
    &self,
    request: &JevvyRequest,
  ) -> impl Future<Output = Result<JevvyResponse, JevvyError>> + Send;
}

/// Sends requests to TypeSafe.ai over HTTPS.
///
/// Busy replies (429, 529) are retried, waiting as long as the API's
/// `Retry-After` header asks, or doubling the wait when it doesn't say.
/// Nothing else is tretried, a timeout included: the API may already
/// have done the work, and one moment gets one call.
///
/// There's deliberately no `Debug` derive as it would print the API key.
#[derive(Clone)]
pub struct Http {
  /// The HTTP client. Cheap to clone and clones share one connecton pool.
  client:      reqwest::Client,
  /// Send as `Authorization: Bearer <api_key>`.
  api_key:     String,
  /// Where requests go, without the `/v1/systemone` path.
  base_url:    String,
  /// Extra tries for busy replies from the cool guys.
  max_retries: u32,
  /// How long one try may take, from connectiong to the last byte.
  timeout:     Duration,
}

impl Http {
  /// A transport with the defaults: the public API, 5 retries,
  /// 30s timeout.
  #[must_use]
  pub fn new(api_key: impl Into<String>) -> Self {
    Http {
      client:      reqwest::Client::new(),
      api_key:     api_key.into(),
      base_url:    DEFAULT_BASE_URL.into(),
      max_retries: DEFAULT_MAX_TRIES,
      timeout:     DEFAULT_TIMEOUT,
    }
  }

  /// Sends to a different server, e.g. a local mock.
  #[must_use]
  pub fn base_url(mut self, base_url: impl Into<String>) -> Self {
    self.base_url = base_url.into();
    self
  }

  /// How many extra tries a busy API gets. `0` means no retries.
  #[must_use]
  pub fn max_retries(mut self, max_retries: u32) -> Self {
    self.max_retries = max_retries;
    self
  }

  /// How long one try may take before it's a [`JevvyError`].
  #[must_use]
  pub fn timeout(mut self, timeout: Duration) -> Self {
    self.timeout = timeout;
    self
  }

  /// Sorts a reqwest error. A timeout becoues our `Timeout`,
  /// anything else stays a `Request` error.
  fn classify(&self, error: reqwest::Error) -> JevvyError {
    if error.is_timeout() {
      JevvyError::Timeout {
        after: self.timeout,
      }
    } else {
      JevvyError::Reqwest(error)
    }
  }
}

impl Transport for Http {
  // `async fn` here writes the future for us to cover the
  // trait's `Send`.
  async fn send(&self, request: &JevvyRequest) -> Result<JevvyResponse, JevvyError> {
    let url = format!(
      "{}/v1/systemone",
      self
        .base_url
        .trim_end_matches('/')
    );
    let mut attempt = 0;

    loop {
      let res = self
        .client
        .post(&url)
        .bearer_auth(&self.api_key)
        .timeout(self.timeout) // per try, moving gives a retry a fresh clock
        .json(request)
        .send()
        .await
        .map_err(|e| self.classify(e))?;

      let status = res
        .status()
        .as_u16();
      let busy = status == 429 || status == 529;

      if busy && attempt < self.max_retries {
        // TypeSafe's API advice first, our doubling guess
        // second and never longer than MAX_WAIT.
        let wait = retry_after(&res)
          .unwrap_or_else(|| backoff(attempt))
          .min(MAX_WAIT);
        tokio::time::sleep(wait).await;
        attempt += 1;
        continue;
      }

      // Out of retries, or not busy. Success or a real error from here.
      let res = check_status(res).await?;

      return res
        .json::<JevvyResponse>()
        .await
        .map_err(|e| self.classify(e));
    }
  }
}

/// How long the APi asked us to wait, from its `Retry-After` header.
///
/// Only the seconds form (`Retry-After: 3`) is read. the date form, or
/// a missing or garbled header, gives `None`, and the caller falls back
/// to [`backoff`].
fn retry_after(res: &reqwest::Response) -> Option<Duration> {
  // Each `?` bails out with None if that step comes up empty.
  let seconds: u64 = res
    .headers()
    .get(RETRY_AFTER)?
    .to_str()
    .ok()?
    .trim()
    .parse()
    .ok()?;

  Some(Duration::from_secs(seconds))
}

/// How long to wait before retry number `attempt` (starting at 0):
/// 0.5s, 1s, 2s, 4s, then 8s from there on.
fn backoff(attempt: u32) -> Duration {
  Duration::from_millis(500 * 2u64.pow(attempt.min(4)))
}

/// Passes a successfull replyu through, or turns a failed one
/// into to matching [`JevvyError`].
async fn check_status(res: reqwest::Response) -> Result<reqwest::Response, JevvyError> {
  let status = res.status();

  if status.is_success() {
    return Ok(res);
  }

  let payload = res
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
    .unwrap_or_else(|| "Unknown API error".to_owned());

  Err(match status.as_u16() {
    | 401 => JevvyError::Unauthorized,
    | 422 => JevvyError::UnprocessableEntity {
      field: payload.and_then(|p| p.field),
      message,
    },
    | 429 => JevvyError::RateLimited,
    | 529 => JevvyError::Overloaded,
    | _ => JevvyError::Api { status, message },
  })
}

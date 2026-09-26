use std::{env, time::Duration};

use indexmap::IndexMap;

use crate::{
  builder::RequestBuilder,
  traits::Rubric,
  types::{
    error::{handle_response, JevvyError},
    request::Request,
    response::Response,
    Entry,
  },
};

const DEFAULT_BASE_URL: &str = "https://api.typesafe.ai";
const DEFAULT_MODEL: &str = "jev-latest";
const DEFAULT_MAX_RETRIES: u32 = 5;

pub struct Jevvy {
  http:        reqwest::Client,
  api_key:     String,
  base_url:    String,
  model:       String,
  max_retries: u32,
}

impl Jevvy {
  pub fn new(api_key: impl Into<String>) -> Self {
    Jevvy {
      http:        reqwest::Client::new(),
      api_key:     api_key.into(),
      base_url:    DEFAULT_BASE_URL.into(),
      model:       DEFAULT_MODEL.into(),
      max_retries: DEFAULT_MAX_RETRIES,
    }
  }

  /// Loads env vars
  ///
  /// # Errors
  ///
  /// [`JevvyError`]
  pub fn from_env() -> Result<Self, JevvyError> {
    let api_key = env::var("TYPESAFE_API_KEY").map_err(|_| JevvyError::MissingApiKey)?;
    let mut client = Self::new(api_key);

    if let Ok(url) = env::var("TYPESAFE_BASE_URL") {
      client = client.base_url(url);
    }

    if let Ok(model) = env::var("TYPESAFE_BASE_MODEL") {
      client = client.model(model);
    }

    if let Some(max) = env::var("TYPESAFE_MAX_RETRIES")
      .ok()
      .and_then(|s| {
        s.parse()
          .ok()
      })
    {
      client = client.max_retries(max);
    }

    Ok(client)
  }

  #[must_use]
  pub fn base_url(mut self, base_url: impl Into<String>) -> Self {
    self.base_url = base_url.into();
    self
  }

  #[must_use]
  pub fn model(mut self, model: impl Into<String>) -> Self {
    self.model = model.into();
    self
  }

  #[must_use]
  pub fn max_retries(mut self, max_retries: u32) -> Self {
    self.max_retries = max_retries;
    self
  }

  /// Sends a hand-build [`Request`] with questions to TypeSafe.ai and returns its answers.
  ///
  /// Most of the time you'll want [`Jevvy::evaluate`] instead, which builds the
  /// request for you . Use this when you already have a `Request`, i.e. one thats
  /// loaded from a file.
  ///
  /// If TypeSafe.ai is busy (429 or 529), waits and tries again,
  /// longer each time, up to `max_retries` extra tries.
  ///
  /// # Errors
  ///
  /// Returns [`JevvyError`] if the request can't be sent, TypeSafe.ai
  /// rejects it, or TypeSafe.ai is still busy after every retry.
  pub async fn execute(&self, request: &Request) -> Result<Response, JevvyError> {
    let url = format!(
      "{}/v1/systemone",
      self
        .base_url
        .trim_end_matches('/')
    );

    let mut attempt = 0;

    loop {
      let res = self
        .http
        .post(&url)
        .bearer_auth(&self.api_key)
        .json(request)
        .send()
        .await?;

      let status = res
        .status()
        .as_u16();
      let busy = status == 429 || status == 529;

      if busy && attempt < self.max_retries {
        tokio::time::sleep(backoff(attempt)).await;
        attempt += 1;
        continue;
      }

      let res = handle_response(res).await?;

      return Ok(
        res
          .json::<Response>()
          .await?,
      );
    }
  }

  /// Starts a new request about `state`, which can be sent text or JSON.
  ///
  /// Add questions with chain methiods, then call `.send()`.
  pub fn evaluate(&self, state: impl Into<Entry>) -> RequestBuilder<'_> {
    let request = Request {
      state:     state.into(),
      model:     self
        .model
        .clone(),
      questions: IndexMap::new(),
    };

    RequestBuilder::new(self, request)
  }

  /// Asks every question in `T`'s rubric about `state`, and fills in a `T`.
  ///
  /// # Errors
  ///
  /// Anything [`Jevvy::execute`] can return , plus a missing, mismatched,
  /// or unknown answer while filling in `T`.
  pub async fn ask<T: Rubric>(&self, state: impl Into<Entry>) -> Result<T, JevvyError> {
    let request = Request {
      state:     state.into(),
      model:     self
        .model
        .clone(),
      // The rubric writes the questions. No builder needed!
      questions: T::questions(),
    };

    let response = self
      .execute(&request)
      .await?;

    T::from_response(&response)
  }
}

/// How long to wait before retry number `attempt` (starting at 0).
/// Doubles each time.
fn backoff(attempt: u32) -> Duration {
  Duration::from_millis(500 * 2u64.pow(attempt.min(4)))
}

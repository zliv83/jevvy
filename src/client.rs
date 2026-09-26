use std::{env, time::Duration};

use crate::{
  builder::JevvyRequestBuilder,
  traits::Rubric,
  transport::{Http, Transport},
  types::{
    jevvy_error::JevvyError, jevvy_request::JevvyRequest, jevvy_response::JevvyResponse,
    questions::Questions, Entry,
  },
};

/// The model every request uses unless you pick another.
pub const DEFAULT_MODEL: &str = "jev-latest";

/// The client. A transport to send through, as well as a defaut model.
///
/// `T` is the plug. it defaults to [`Http`], so plain 'Jevvy' means
/// "talks to the real API".
#[derive(Clone)]
pub struct Jevvy<T = Http> {
  /// How requests reach the API
  transport: T,
  /// The model new requests start with.
  model:     String,
}

// -- Only for the real API (T = HTTP) --

impl Jevvy<Http> {
  /// A client for the real API, with the default model, retries and timeout.
  #[must_use]
  pub fn new(api_key: impl Into<String>) -> Self {
    Jevvy {
      transport: Http::new(api_key),
      model:     DEFAULT_MODEL.into(),
    }
  }

  /// A client configured from environment variables.
  ///
  /// `TYPESAFE_API_KEY` is required. `TYPESAFE_BASE_URL`,
  /// `TYPESAFE_BASE_MODEL` and `TYPESAFE_MAX_RETRIES` are optional.
  ///
  /// # Errors
  ///
  /// [`JevvyError::MissingApiKey`] if `TYPESAFE_API_KEY` isn't set.
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

  /// Sends to a diverent server, e.g. a local mock.
  #[must_use]
  pub fn base_url(mut self, base_url: impl Into<String>) -> Self {
    self.transport = self
      .transport
      .base_url(base_url);
    self
  }

  /// How many extra tries a busy API (429, 529) gets.
  #[must_use]
  pub fn max_retries(mut self, max_retries: u32) -> Self {
    self.transport = self
      .transport
      .max_retries(max_retries);
    self
  }

  /// How long one try may take before it's a [`JevvyError::Timeout`].
  #[must_use]
  pub fn timeout(mut self, timeout: Duration) -> Self {
    self.transport = self
      .transport
      .timeout(timeout);
    self
  }
}

// -- For every transport --

impl<T: Transport> Jevvy<T> {
  /// Swaps the plug, keeping the model.
  ///
  /// `U` is the new transport's type, so the result is a `Jevy<U>`.
  #[must_use]
  pub fn with_transport<U: Transport>(self, transport: U) -> Jevvy<U> {
    Jevvy {
      transport,
      model: self.model,
    }
  }

  /// The model new reqeusts start with.
  #[must_use]
  pub fn model(mut self, model: impl Into<String>) -> Self {
    self.model = model.into();
    self
  }

  /// Checks every question, then sends a finished [`JevvyRequest`].
  ///
  /// This is the one door every request goes through. Nothing leaves
  /// the machine until every question is within the API's limits.
  ///
  /// # Errors
  ///
  /// [`JevvyError::BadOptionCount`] or [`JevvyError::BadLevelCount`]
  /// before sending, or anything the transport returns.
  pub async fn send(&self, jevvy_request: &JevvyRequest) -> Result<JevvyResponse, JevvyError> {
    // `?` stops at the first bad question and returns its error.
    for (key, question) in &jevvy_request.questions {
      question.validate(key)?;
    }

    self
      .transport
      .send(jevvy_request)
      .await
  }

  /// Starts a new request about `state`, which can be text or JevvyResponse.
  ///
  /// Add questions with the chain methids, then call `.send()`.
  pub fn evaluate(&self, state: impl Into<Entry>) -> JevvyRequestBuilder<'_, T> {
    let jevvy_request = JevvyRequest {
      state:     state.into(),
      model:     self
        .model
        .clone(),
      questions: Questions::new(),
    };

    JevvyRequestBuilder::new(self, jevvy_request)
  }

  /// Asks every question in the `R`'s rubric about `state`, and fillls in an `R`.
  ///
  /// # Errors
  ///
  /// Antything [`Jevvy::send`] can return, plus a missing, mismatched,
  /// or unknown asnwer while filling in `R`.
  pub async fn ask<R: Rubric>(&self, state: impl Into<Entry>) -> Result<R, JevvyError> {
    let jevvy_request = JevvyRequest {
      state:     state.into(),
      model:     self
        .model
        .clone(),
      questions: R::questions(),
    };

    let jevvy_response = self
      .send(&jevvy_request)
      .await?;

    R::from_response(&jevvy_response)
  }
}

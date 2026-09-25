use crate::{
  client::Jevvy,
  types::{
    error::JevvyError,
    questions::{ChoiceOption, Question},
    request::Request,
    response::Response,
    Entry,
  },
};

/// A request being put together, one question at a time.
///
/// Mady by [`Jevvy::evaluate`]. Add questions with the chain methods,
/// then call `.send()`.
#[must_use = "a RequestBuilder does nothing until you call .send()"]
pub struct RequestBuilder<'a> {
  /// The client that will send this request. Borrowed, not owned. 🍸
  client: &'a Jevvy,

  /// The request being filled in.
  request: Request,
}

impl<'a> RequestBuilder<'a> {
  /// Starts a builder.
  pub(crate) fn new(client: &'a Jevvy, request: Request) -> Self {
    Self { client, request }
  }

  /// Adds any question under `key`. This is a shared helper
  pub fn question(mut self, key: impl Into<String>, question: Question) -> Self {
    self
      .request
      .questions
      .insert(key.into(), question);
    self
  }

  /// Adds a yes/no question under `key`.
  ///
  /// The answer comes back as the porbability of "yes", from 0 to 1.
  pub fn noul(self, key: impl Into<String>, instructions: impl Into<Entry>) -> Self {
    self.question(key, Question::noul(instructions))
  }

  /// Adds a question that rates the state on an ordered scale.
  pub fn score(
    self,
    key: impl Into<String>,
    instructions: impl Into<Entry>,
    levels: impl IntoIterator<Item = impl Into<Entry>>,
  ) -> Self {
    self.question(key, Question::score(instructions, levels))
  }

  /// Adds a question that picks one option from a set
  pub fn choice(
    self,
    key: impl Into<String>,
    instructions: impl Into<Entry>,
    options: impl IntoIterator<Item = impl Into<ChoiceOption>>,
  ) -> Self {
    self.question(key, Question::choice(instructions, options))
  }

  /// Uses a different model for this request only
  pub fn model(mut self, model: impl Into<String>) -> Self {
    self
      .request
      .model = model.into();
    self
  }

  /// Sends the request and returns TypeSafe.ai's answers.
  ///
  /// # Errors
  ///
  /// Same as [`Jevvy::execute`]: network trouble, a rejected reqeust
  /// or TypeSafe.ai is still busy after every retry.
  pub async fn send(self) -> Result<Response, JevvyError> {
    self
      .client
      .execute(&self.request)
      .await
  }
}

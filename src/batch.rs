use std::fmt::Display;

use crate::{
  client::Jevvy,
  sheet::QuestionSheet,
  traits::{Form, Levels, Options},
  transport::{Http, Transport},
  types::{
    jevvy_error::JevvyError, jevvy_request::JevvyRequest, jevvy_response::JevvyResponse,
    questions::Question, Entry,
  },
};

/// One request being put together.
/// One state, and every question about this moment, from any number
/// of forms and loose questions.
///
/// Made by [`Jevvy::ask`]. Nothing is sent until `.send()`.
#[must_use = "a Batch does nothing until you call .send()"]
pub struct Batch<'a, T = Http> {
  /// The client that will send this request. Borrowed, not owned. 🍸
  client: &'a Jevvy<T>,

  /// The request being filled in.
  jevvy_request: JevvyRequest,
}

impl<'a, T: Transport> Batch<'a, T> {
  /// Starts a batch.
  pub(crate) fn new(client: &'a Jevvy<T>, jevvy_request: JevvyRequest) -> Self {
    Self {
      client,
      jevvy_request,
    }
  }

  /// A root sheet (no prefix) over this request's questions.
  /// Every chain method below writes through one.
  fn sheet(&mut self) -> QuestionSheet<'_> {
    QuestionSheet::root(
      &mut self
        .jevvy_request
        .questions,
    )
  }

  /// Adds any question under `key`.
  pub fn question(mut self, key: &str, question: Question) -> Self {
    self
      .sheet()
      .question(key, question);
    self
  }

  /// Adds a true/false question under `key`.
  pub fn noul(mut self, key: &str, instructions: impl Into<Entry>) -> Self {
    self
      .sheet()
      .noul(key, instructions);
    self
  }

  /// Adds a true/false question with both sides spelled out.
  pub fn noul_with(
    mut self,
    key: &str,
    instructions: impl Into<Entry>,
    when_true: impl Into<Entry>,
    when_false: impl Into<Entry>,
  ) -> Self {
    self
      .sheet()
      .noul_with(key, instructions, when_true, when_false);
    self
  }

  /// Adds a Choice over every variant of `O`.
  pub fn choice<O: Options>(mut self, key: &str, instructions: impl Into<Entry>) -> Self {
    self
      .sheet()
      .choice::<O>(key, instructions);
    self
  }

  /// Adds a Score over every level of `L`, lowest first.
  pub fn score<L: Levels>(mut self, key: &str, instructions: impl Into<Entry>) -> Self {
    self
      .sheet()
      .score::<L>(key, instructions);
    self
  }

  /// Adds every question in `form`, under `prefix`.
  pub fn form<F: Form>(mut self, prefix: &str, form: &F) -> Self {
    self
      .sheet()
      .form(prefix, form);
    self
  }

  /// Adds one form per `(id, form)` pair, each under `<prefix>.<id>`.
  pub fn each<Id: Display, F: Form>(
    mut self,
    prefix: &str,
    forms: impl IntoIterator<Item = (Id, F)>,
  ) -> Self {
    self
      .sheet()
      .each(prefix, forms);
    self
  }

  /// Uses a different model for this request only.
  pub fn model(mut self, model: impl Into<String>) -> Self {
    self
      .jevvy_request
      .model = model.into();
    self
  }

  /// The request as built so far. Print it to see exactly what will go out.
  #[must_use]
  pub fn request(&self) -> &JevvyRequest {
    &self.jevvy_request
  }

  /// Sends the request and returns TypeSafe.ai's answers.
  ///
  /// # Errors
  ///
  /// Same as [`Jevvy::send`]: a question over the API's limits, network
  /// trouble, a rejected request, or TypeSafe.ai is still busy after every retry.
  pub async fn send(self) -> Result<JevvyResponse, JevvyError> {
    self
      .client
      .send(&self.jevvy_request)
      .await
  }
}

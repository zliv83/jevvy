use crate::{
  choice::ChoiceQuestion,
  constants::BASE_MODEL,
  error::JevvyError,
  noul::NoulQuestion,
  reply::AnyAnswer,
  score::ScoreQuestion,
  types::{QuestionId, State},
};
use serde::Serialize;
use std::collections::HashMap;

/// A question you can ask Jev.
///
/// Each question type has a matching answer type.
pub trait Question {
  type Answer;

  /// Returns the question's ID, used in both the request and Jev's reply.
  fn id(&self) -> QuestionId;

  /// Borrows the answer from `AnyAnswer` if its kind matches this question type.
  /// Returns `None` if the kinds don't match.
  fn extract(answer: &AnyAnswer) -> Option<&Self::Answer>;
}

/// A Noul, Choice, or Score question.
///
/// When sent to Jev, the question's `"type"` is its variant name in lowercase.
/// Renaming a variant changes the value Jev receives.
#[derive(Clone, Debug, Serialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum AnyQuestion {
  Noul(NoulQuestion),
  Choice(ChoiceQuestion),
  Score(ScoreQuestion),
}

impl Question for AnyQuestion {
  type Answer = AnyAnswer;

  fn id(&self) -> QuestionId {
    match self {
      | Self::Noul(question) => question.id(),
      | Self::Choice(question) => question.id(),
      | Self::Score(question) => question.id(),
    }
  }

  fn extract(answer: &AnyAnswer) -> Option<&Self::Answer> {
    Some(answer)
  }
}

impl From<NoulQuestion> for AnyQuestion {
  fn from(value: NoulQuestion) -> Self {
    Self::Noul(value)
  }
}

impl From<ChoiceQuestion> for AnyQuestion {
  fn from(value: ChoiceQuestion) -> Self {
    Self::Choice(value)
  }
}

impl From<ScoreQuestion> for AnyQuestion {
  fn from(value: ScoreQuestion) -> Self {
    Self::Score(value)
  }
}

/// A set of questions about the same state, sent to Jev in one request.
///
/// Build it with `Questions::builder()` and send it with `jev.ask(&questions)`.
#[derive(Debug, Serialize)]
#[expect(clippy::struct_field_names, reason = "wire field is named `questions`")]
pub struct Questions {
  model:     &'static str,
  state:     State,
  questions: HashMap<QuestionId, AnyQuestion>,
}

impl Questions {
  #[must_use]
  pub const fn builder() -> QuestionsBuilder {
    QuestionsBuilder {
      state:     None,
      questions: Vec::new(),
    }
  }
}

#[derive(Debug)]
pub struct QuestionsBuilder {
  state:     Option<State>,
  questions: Vec<AnyQuestion>,
}

impl QuestionsBuilder {
  #[must_use]
  pub fn state(mut self, state: impl Into<State>) -> Self {
    self.state = Some(state.into());
    self
  }

  /// Adds a clone of `question` with the same ID.
  ///
  /// Keep the original to look up its answer with `reply.answer(&question)`.
  #[must_use]
  pub fn question<Q>(mut self, question: &Q) -> Self
  where
    Q: Clone + Into<AnyQuestion>,
  {
    self
      .questions
      .push(
        question
          .clone()
          .into(),
      );
    self
  }

  pub fn build(self) -> Result<Questions, JevvyError> {
    let state = self
      .state
      .ok_or(JevvyError::MissingState)?;

    if self
      .questions
      .is_empty()
    {
      return Err(JevvyError::NoQuestions);
    }

    let mut questions = HashMap::with_capacity(
      self
        .questions
        .len(),
    );

    for question in self.questions {
      let id = question.id();

      if questions
        .insert(id, question)
        .is_some()
      {
        return Err(JevvyError::DuplicateQuestion { id });
      }
    }

    Ok(Questions {
      model: BASE_MODEL,
      state,
      questions,
    })
  }
}

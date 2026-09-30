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
/// Each question type names the answer Jev gives back for it.
pub trait Question {
  type Answer;

  /// Identifies this question in the questions you send and in Jev's reply.
  fn id(&self) -> QuestionId;

  /// Picks this question's kind of answer out of `AnyAnswer`, or `None`
  /// if Jev answered with a different kind.
  fn extract(answer: &AnyAnswer) -> Option<&Self::Answer>;
}

/// Any kind of question, in its wire shape. The `"type"` field comes from
/// the variant names, so renaming the variant changes what Jev receives.
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

/// The questions you ask Jev together, all about one shared state.
///
/// Construct with `Questions::builder()`, send with `jev.ask(&questions)`.
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

  /// Adds a copy of `question`. Keep the original: it's how you look up
  /// this question's answer.
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

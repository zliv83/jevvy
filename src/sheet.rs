//! Sheets: where forms write their questions and read their answers.
//!
//! A [`QuestionSheet`] is a clipboard over a request's questons.
//! An [`AnswerSheet`] is the same clipboard of a reply's answer.

use std::fmt::Display;

use crate::{
  traits::{Form, Levels, Options},
  types::{
    answers::{Answer, Answers},
    jevvy_error::JevvyError,
    questions::{Question, Questions},
    typed::{Choice, Score},
    Entry,
  },
};

/// Joins a prefix and a local key with a dot: `"triage"` + `"team"`
/// is `"triage.team"`.
fn join(prefix: &str, local: &str) -> String {
  match (prefix.is_empty(), local.is_empty()) {
    | (true, _) => local.to_owned(),
    | (_, true) => prefix.to_owned(),
    | _ => format!("{prefix}.{local}"),
  }
}

/// Builds the "wrong kind of answer" error.
fn wrong_type(key: &str, expected: &'static str, found: &Answer) -> JevvyError {
  JevvyError::WrongAnswerType {
    key: key.to_owned(),
    expected,
    found: found.kind(),
  }
}

// -- Writing --

/// A clipboard over a request's questions.
///
/// Forms write onto the sheet they are handed. The never own
/// the request. Every writer reutnrs a sheet, so calls can chain.
pub struct QuestionSheet<'a> {
  /// Added in front of every key written.
  prefix:    String,
  /// The request's questons, borrowed while the sheet is in use.
  questions: &'a mut Questions,
}

impl<'a> QuestionSheet<'a> {
  /// A sheet with no prefix, writing straight into `questions`.
  pub fn root(questions: &'a mut Questions) -> Self {
    QuestionSheet {
      prefix: String::new(),
      questions,
    }
  }

  /// The full key for a local question. `key("team")` is `"triage.team"`.
  #[must_use]
  pub fn key(&self, local: &str) -> String {
    join(&self.prefix, local)
  }

  /// Writes any question under `key`. A second queston under thei same
  /// full key replaces the first.
  pub fn question(&mut self, key: &str, question: Question) -> &mut Self {
    let full = self.key(key);
    self
      .questions
      .insert(full, question);
    self
  }

  /// A true/false question. The answer is the probability of true.
  pub fn noul(&mut self, key: &str, instructions: impl Into<Entry>) -> &mut Self {
    self.question(key, Question::noul(instructions))
  }

  /// A true/false question with both sides spelled out.
  pub fn noul_with(
    &mut self,
    key: &str,
    instructions: impl Into<Entry>,
    when_true: impl Into<Entry>,
    when_false: impl Into<Entry>,
  ) -> &mut Self {
    self.question(
      key,
      Question::noul_with(instructions, when_true, when_false),
    )
  }

  /// A choice over every variant of `T`.
  pub fn choice<T: Options>(&mut self, key: &str, instructions: impl Into<Entry>) -> &mut Self {
    self.question(key, T::question(instructions))
  }

  /// A Score over every level of `T`, lowest first.
  pub fn score<T: Levels>(&mut self, key: &str, instructions: impl Into<Entry>) -> &mut Self {
    self.question(key, T::question(instructions))
  }

  /// Lets `form` write it's questions under `key`: its `"team"` lands
  /// here as `"key.team"`.
  pub fn form<F: Form>(&mut self, key: &str, form: &F) -> &mut Self {
    // A child sheet. Same questions but with a longer prefix.
    let mut child = QuestionSheet {
      prefix:    self.key(key),
      questions: &mut *self.questions,
    };
    form.ask(&mut child);
    self
  }

  /// One form per `(id, form)` pair, each ucer `<prefix>,<id>`.
  ///
  /// This is how questions get stamped out of data.
  /// One `SamePerson` per candidate record, all in the same request.
  pub fn each<Id: Display, F: Form>(
    &mut self,
    prefix: &str,
    forms: impl IntoIterator<Item = (Id, F)>,
  ) -> &mut Self {
    for (id, form) in forms {
      self.form(&join(prefix, &id.to_string()), &form);
    }
    self
  }
}

// -- Reading -- (its good for you 📚)

///  Clipboard over a reply's answers.
///
/// The mirror of [`QuestionSheet`]: A form reads bak with the same
/// short keys it wrote with.
pub struct AnswerSheet<'a> {
  /// Added in front of every key read here.
  prefix:  String,
  /// The reply's answers, borrowed.
  answers: &'a Answers,
}

impl<'a> AnswerSheet<'a> {
  /// A sheet with no prefix, reading straight from `answers`.
  #[must_use]
  pub fn root(answers: &'a Answers) -> Self {
    AnswerSheet {
      prefix: String::new(),
      answers,
    }
  }

  /// The full key for a local one: `key("team")` is `"triage.team"`.
  #[must_use]
  pub fn key(&self, local: &str) -> String {
    join(&self.prefix, local)
  }

  /// Finds the answer under a full key.
  fn find(&self, full: &str) -> Result<&'a Answer, JevvyError> {
    self
      .answers
      .get(full)
      .ok_or_else(|| JevvyError::MissingAnswer(full.to_owned()))
  }

  /// Any answer under `key`, exactly as it came off the wire.
  ///
  /// # Errors
  ///
  /// [`JevvyError::MissingAnswer`] if nothing came back under `key`.
  pub fn raw(&self, key: &str) -> Result<&'a Answer, JevvyError> {
    self.find(&self.key(key))
  }

  /// The probability of true for the Noul under `key`.
  ///
  /// # Errors
  ///
  /// Missing, or not a Noul.
  pub fn noul(&self, key: &str) -> Result<f64, JevvyError> {
    let full = self.key(key);
    match self.find(&full)? {
      | Answer::Noul(a) => Ok(a.noul),
      | other => Err(wrong_type(&full, "noul", other)),
    }
  }

  /// The Choice under `key`, read into your enum `T`.
  ///
  /// # Errors
  ///
  /// Missing, not a Choice, or an option `T` doesn't have.
  pub fn choice<T: Options>(&self, key: &str) -> Result<Choice<T>, JevvyError> {
    let full = self.key(key);
    match self.find(&full)? {
      | Answer::Choice(a) => Choice::from_answer(&full, a),
      | other => Err(wrong_type(&full, "choice", other)),
    }
  }

  /// The Score under `key`, read onto your levels `T`.
  ///
  /// # Errors
  ///
  /// Missing, not a Score, or a level position `T` doesn't have.
  pub fn score<T: Levels>(&self, key: &str) -> Result<Score<T>, JevvyError> {
    let full = self.key(key);
    match self.find(&full)? {
      | Answer::Score(a) => Score::from_answer(&full, a),
      | other => Err(wrong_type(&full, "score", other)),
    }
  }

  /// Lets `form` read its answers from under `key`.
  ///
  /// # Errors
  ///
  /// Whatever the form's `read` returns.
  pub fn form<F: Form>(&self, key: &str, form: &F) -> Result<F::Answers, JevvyError> {
    let child = AnswerSheet {
      prefix:  self.key(key),
      answers: self.answers,
    };
    form.read(&child)
  }

  /// Reads back what [`QuestionSheet::each`] wrote.
  /// One answer set per `(id, form)` pair, in the same order
  ///
  /// # Errors
  ///
  /// The first form that fails to read.
  pub fn each<Id: Display, F: Form>(
    &self,
    prefix: &str,
    forms: impl IntoIterator<Item = (Id, F)>,
  ) -> Result<Vec<(Id, F::Answers)>, JevvyError> {
    forms
      .into_iter()
      .map(|(id, form)| {
        let answers = self.form(&join(prefix, &id.to_string()), &form)?;
        Ok((id, answers))
      })
      .collect()
  }
}

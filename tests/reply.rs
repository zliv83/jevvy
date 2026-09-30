mod common;

use std::error::Error;

use common::reply_to;
use jevvy::{ChoiceQuestion, JevvyError, NoulQuestion, ScoreQuestion};

fn noul_question() -> Result<NoulQuestion, JevvyError> {
  NoulQuestion::builder()
    .instructions("Does this need attention?")
    .build()
}

#[test]
fn answers_a_noul_question() -> Result<(), Box<dyn Error>> {
  let question = noul_question()?;
  let reply = reply_to(&question, "noul")?;

  let answer = reply.answer(&question)?;

  assert!(
    (answer
      .confidence()
      .value()
      - 0.92)
      .abs()
      < f64::EPSILON
  );
  assert_eq!(
    reply
      .usage()
      .input_tokens(),
    42
  );
  Ok(())
}

#[test]
fn answers_a_choice_question() -> Result<(), Box<dyn Error>> {
  let question = ChoiceQuestion::builder()
    .instructions("Which team should handle this?")
    .option(
      "billing",
      "Payments, invoicing, refunds",
    )
    .option(
      "technical",
      "Bugs, outages, integrations",
    )
    .option_without_description("other")
    .build()?;
  let reply = reply_to(&question, "choice")?;

  let answer = reply.answer(&question)?;

  assert_eq!(answer.choice(), "technical");
  assert!(answer
    .probabilities()
    .iter()
    .any(|(option, probability)| {
      option == "technical" && (probability - 0.85).abs() < f64::EPSILON
    }));
  assert!(
    (answer
      .confidence()
      .value()
      - 0.80)
      .abs()
      < f64::EPSILON
  );
  Ok(())
}

#[test]
fn answers_a_score_question() -> Result<(), Box<dyn Error>> {
  let question = ScoreQuestion::builder()
    .instructions("How urgent is this?")
    .level("Can wait")
    .level("Needs attention this week")
    .level("Needs attention today")
    .build()?;
  let reply = reply_to(&question, "score")?;

  let answer = reply.answer(&question)?;

  assert!((answer.score() - 1.6).abs() < f64::EPSILON);
  assert!(answer
    .legend()
    .iter()
    .any(|(level, description)| level == "2" && description == "Needs attention today"));
  assert!(answer
    .probabilities()
    .iter()
    .any(|(level, probability)| level == "2" && (probability - 0.7).abs() < f64::EPSILON));
  Ok(())
}

#[test]
fn a_question_missing_from_the_reply_is_an_error() -> Result<(), Box<dyn Error>> {
  let asked = noul_question()?;
  let never_asked = noul_question()?;
  let reply = reply_to(&asked, "noul")?;

  assert!(matches!(
    reply.answer(&never_asked),
    Err(JevvyError::MissingAnswer { .. })
  ));
  Ok(())
}

#[test]
fn an_answer_of_the_wrong_kind_is_an_error() -> Result<(), Box<dyn Error>> {
  let question = noul_question()?;
  let reply = reply_to(&question, "choice")?;

  assert!(matches!(
    reply.answer(&question),
    Err(JevvyError::WrongAnswerKind { .. })
  ));
  Ok(())
}

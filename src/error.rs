use crate::types::QuestionId;
use thiserror::Error;

/// Jevvy Errors
#[derive(Debug, Error)]
pub enum JevvyError {
  /// Reqwest Errors
  #[error("HTTP exchange failed: {0}")]
  Request(#[from] reqwest::Error),

  /// TypeSafe API Errors
  #[error("TypeSafe returned HTTP {status}: {body}")]
  API {
    status: reqwest::StatusCode,
    body:   String,
  },

  /// Decoding error via serde_json
  #[error("Could not decode Jev's response: {0}")]
  Decode(#[from] serde_json::Error),

  /// Missing api_key
  #[error("Could not read env var: {0}")]
  MissingApiKey(#[from] std::env::VarError),

  /// .env file could not be loaded
  #[cfg(feature = "dotenvy")]
  #[error("Could not load .env: {0}")]
  Dotenv(#[from] dotenvy::Error),

  // -- SDK Errors ---
  /// Missing instructions
  #[error("question instructions are required")]
  MissingInstructions,

  /// Zero or more than 255 options
  #[error("choice requires 1 to 255 options - received {count}")]
  InvalidChoiceOptionCount { count: usize },

  /// Less than 2 or more than 10 levels.
  #[error("score requires 2 to 10 levels - received {count}")]
  InvalidScoreLevelCount { count: usize },

  /// Missing State
  #[error("question state is missing")]
  MissingState,

  /// No questions are present
  #[error("at least one question is required")]
  NoQuestions,

  /// Duplicate questions by id
  #[error("duplicate questions are not allowed, the id for the questions is: {id}")]
  DuplicateQuestion { id: QuestionId },

  /// The reply has no answer for this question
  #[error("the reply has no answer for question {id}")]
  MissingAnswer { id: QuestionId },

  /// Jev's answer is a different kind than the question asked for
  #[error("the answer to question {id} is a different kind than the question")]
  WrongAnswerKind { id: QuestionId },
}

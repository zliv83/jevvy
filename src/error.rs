use crate::types::QuestionId;
use thiserror::Error;

/// Errors from building questions, calling Jev, or reading answers.
#[derive(Debug, Error)]
pub enum JevvyError {
  /// The HTTP request or response failed.
  #[error("HTTP exchange failed: {0}")]
  Request(#[from] reqwest::Error),

  /// TypeSafe returned an HTTP status outside the 2xx range.
  #[error("TypeSafe returned HTTP {status}: {body}")]
  API {
    status: reqwest::StatusCode,
    body:   String,
  },

  /// Jev's response could not be decoded with `serde_json`.
  #[error("Could not decode Jev's response: {0}")]
  Decode(#[from] serde_json::Error),

  /// The API key environment variable is missing or is not valid Unicode.
  #[error("Could not read env var: {0}")]
  MissingApiKey(#[from] std::env::VarError),

  /// The `.env` file could not be loaded.
  #[cfg(feature = "dotenvy")]
  #[error("Could not load .env: {0}")]
  Dotenv(#[from] dotenvy::Error),

  // -- SDK Errors ---
  /// The question has no instructions.
  #[error("question instructions are required")]
  MissingInstructions,

  /// The choice question has no options or more than 255.
  #[error("choice requires 1 to 255 options - received {count}")]
  InvalidChoiceOptionCount { count: usize },

  /// The score question has fewer than 2 levels or more than 10.
  #[error("score requires 2 to 10 levels - received {count}")]
  InvalidScoreLevelCount { count: usize },

  /// The request has no state.
  #[error("question state is missing")]
  MissingState,

  /// The request has no questions.
  #[error("at least one question is required")]
  NoQuestions,

  /// Two or more questions in the request share an ID.
  #[error("duplicate questions are not allowed, the id for the questions is: {id}")]
  DuplicateQuestion { id: QuestionId },

  /// The reply has no answer for this question.
  #[error("the reply has no answer for question {id}")]
  MissingAnswer { id: QuestionId },

  /// The answer's kind does not match the question type.
  #[error("the answer to question {id} is a different kind than the question")]
  WrongAnswerKind { id: QuestionId },
}

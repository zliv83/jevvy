//! Manual example: builds a `Request` by hand and sends it with `execute`.
//!
//! Most code should use `jevvy.evaluate(...)` instead (see smoke.rs).
//! This shows the do-it-by-hand route.
//!
//! Run with: cargo run --example manual

use indexmap::IndexMap;
use jevvy::{
	client::Jevvy,
	types::{questions::Question, request::Request},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
  dotenvy::dotenv()?;
  // Build the client
  let client = Jevvy::from_env()?;

  // Build one yes/no question
  let mut questions = IndexMap::new();
  questions.insert(
    "is_urgent".to_string(),
    Question::Noul {
      instructions: "Does this convey urgency?".into(),
      criteria:     None,
    },
  );

  // Wrap it in a request.
  let request = Request {
    state: "Help! My payouts have been failing for 3 days".into(),
    model: "jev-latest".into(),
    questions,
  };

  // Print exactly what we're about to send
  println!("sending:\n{}\n", serde_json::to_string_pretty(&request)?);

  // Send it and print what comes back
  let response = client
    .execute(&request)
    .await?;
  println!("got back:\n{response:#?}");

  Ok(())
}

use jevvy::{
	traits::Options,
	types::{answers::ChoiceAnswer, typed::Choice},
};
use serde_json::json;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Area {
  Networking,
  Serde,
  Docs,
}

impl Options for Area {
  const ALL: &'static [Self] = &[Area::Networking, Area::Serde, Area::Docs];

  fn name(&self) -> &'static str {
    match self {
      | Area::Networking => "networking",
      | Area::Serde => "serde",
      | Area::Docs => "docs",
    }
  }

  fn describe(&self) -> Option<&'static str> {
    match self {
      | Area::Networking => Some("HTTP client, retries, timeouts"),
      | Area::Serde => Some("Request/response types, JSON shape"),
      | Area::Docs => Some("README, examples, docs.rs"),
    }
  }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
  // A fake answer, shaped like the one the API sends back.
  let answer: ChoiceAnswer = serde_json::from_value(json!({
    "choice": "serde",
    "confidence": 0.82,
    "probabilities": { "networking": 0.10, "serde": 0.82, "docs": 0.08 },
  }))?;

  let area = Choice::<Area>::try_from(&answer)?;
  println!("{area:?}");
  println!("{:?}", area.confident(0.9)); // None: 0.82 isn't sure enough
  println!("{:?}", area.confident(0.8)); // Some(Serde)

  // One the bouncer should turn away.
  let pizza: ChoiceAnswer = serde_json::from_value(json!({
    "choice": "pizza",
    "confidence": 0.9,
    "probabilities": { "pizza": 0.9 },
  }))?;

  match Choice::<Area>::try_from(&pizza) {
    | Ok(_) => println!("uh oh, pizza got in"),
    | Err(e) => println!("{e}"),
  }

  Ok(())
}

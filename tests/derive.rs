//! The derive macros write exactly what the hand writes.
//!
//! One form typed by hand, the same form derived, and the request JSON
//! each on produces compared byte for byte.

use jevvy::{traits::Options, types::Entry};
use serde_json::json;

// -- By Hand --

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Team {
  Billing,
  Orders,
  Other,
}

impl Options for Team {
  const ALL: &'static [Self] = &[Team::Billing, Team::Orders, Team::Other];

  fn name(&self) -> &'static str {
    match self {
      | Team::Billing => "billing",
      | Team::Orders => "orders",
      | Team::Other => "none_of_the_above",
    }
  }

  fn describe(&self) -> Option<Entry> {
    match self {
      | Team::Billing => Some(Entry::from(json!({
        "what": "Charges, invoices, refunds, or subscriptions",
        "not_for": "Order tracking or account access",
        "examples": ["I was charged twice", "Where is my refund?"],
      }))),
      | Team::Orders => Some(Entry::from(
        "Order status, delivery, cancellation, or returns",
      )),
      | Team::Other => None,
    }
  }
}

// -- Derived --

mod derived {
  // One import brings the trait *and* the derive (they live in
  // different namespaces, so the same name holds both).
  use jevvy::traits::Options;

  #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Options)]
  pub enum Team {
    #[describe(
      what = "Charges, invoices, refunds, or subscriptions",
      not_for = "Order tracking or account access",
      examples = ["I was charged twice", "Where is my refund?"]
    )]
    Billing,
    #[describe("Order status, delivery, cancellation, or returns")]
    Orders,
    #[name("none_of_the_above")]
    Other,
  }
}

#[test]
fn derived_options_match_hand_written() {
  for (hand, derived) in Team::ALL
    .iter()
    .zip(derived::Team::ALL)
  {
    assert_eq!(hand.name(), derived.name());
    assert_eq!(
      serde_json::to_string(&hand.describe()).unwrap(),
      serde_json::to_string(&derived.describe()).unwrap(),
    );
  }
  assert_eq!(Team::ALL.len(), derived::Team::ALL.len());
  assert_eq!(
    derived::Team::from_name("none_of_the_above"),
    Some(derived::Team::Other)
  );
}

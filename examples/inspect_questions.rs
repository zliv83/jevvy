use jevvy::{ChoiceQuestion, JevvyError, NoulQuestion, Questions, ScoreQuestion};

#[expect(
  clippy::print_stdout,
  reason = "printing the JSON is the point of this example"
)]
fn main() -> Result<(), JevvyError> {
  let manufacturing_investment = NoulQuestion::builder()
    .instructions("Does this article describe an investment in manufacturing?")
    .criteria(
      "Building or expanding manufacturing",
      "An event unrelated to manufacturing investment",
    )
    .build()?;

  let event_type = ChoiceQuestion::builder()
    .instructions("What event does this article primarily report?")
    .option(
      "expansion",
      "New facilities, capacity, or markets",
    )
    .option(
      "partnership",
      "A deal or alliance with another company",
    )
    .option_without_description("other")
    .build()?;

  let materiality = ScoreQuestion::builder()
    .instructions("How significant is the event for Apple's operations?")
    .level("Routine activity")
    .level("Meaningful operational change")
    .level("Major operational change")
    .build()?;

  let questions = Questions::builder()
    .state("Apple announced a new manufacturing facility.")
    .question(&manufacturing_investment)
    .question(&event_type)
    .question(&materiality)
    .build()?;

  let json = serde_json::to_string_pretty(&questions)?;

  println!("{json}");

  Ok(())
}

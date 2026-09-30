use jevvy::{Jev, JevvyError, NoulQuestion, Questions};

#[tokio::main]
#[expect(
  clippy::print_stdout,
  reason = "printing Jev's answer is the point of this example"
)]
async fn main() -> Result<(), JevvyError> {
  let about_apple = NoulQuestion::builder()
    .instructions("Does this article concern Apple?")
    .build()?;

  let questions = Questions::builder()
    .state("Apple announced a new manufacturing facility in Texas.")
    .question(&about_apple)
    .build()?;

  let jev = Jev::from_env()?;
  let reply = jev
    .ask(&questions)
    .await?;

  let relevance = reply.answer(&about_apple)?;

  println!("{}", relevance.confidence());

  Ok(())
}

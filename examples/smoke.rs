use jevvy::client::Jevvy;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
  dotenvy::dotenv()?;
  // Build the client
  let jevvy = Jevvy::from_env()?;

  let res = jevvy
    .evaluate("Help! My payouts have been failing for 3 days")
    .noul("is_urgent", "Does this convey urgency")
    .choice(
      "team",
      "Which team should handle this?",
      [
        ("billing", "Payments, invoicing, refunds"),
        ("technical", "Bugs, outages, integrations"),
      ],
    )
    .score(
      "mood",
      "How frustrated is the customer?",
      ["Calm", "Frustrated", "Very Angry"],
    )
    .send()
    .await?;

  println!("urgent: {:.2}", res.noul("is_urgent")?);
  let team = res.choice("team")?;
  println!(
    "team: {} ({:.0}% sure)",
    team.choice,
    team.confidence * 100.0
  );
  println!(
    "mood: {:.2}",
    res
      .score("mood")?
      .score
  );
  Ok(())
}

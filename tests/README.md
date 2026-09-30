# Answer fixtures

`fixtures/noul.json`, `fixtures/choice.json`, and `fixtures/score.json` each hold
one TypeSafe answer, one per answer type. They are synthetic examples based on the
[official SDK's response types](https://github.com/typesafe-ai/typesafe-sdk-js/blob/main/src/types.ts),
not captured API responses. Tests using them need neither an API key nor network
access.

Question ids come from a global counter, so a fixture can't know the id of the
question it answers. `common::reply_to` wraps a fixture in a `Reply` keyed by the
real `question.id()`:

```rust
mod common;

#[test]
fn my_test() -> Result<(), Box<dyn std::error::Error>> {
  let question = jevvy::ChoiceQuestion::builder()
    .instructions("Which team should handle this?")
    .option_without_description("technical")
    .build()?;
  let reply = common::reply_to(&question, "choice")?;
  let answer = reply.answer(&question)?; // &ChoiceAnswer
  Ok(())
}
```

Run the fixture tests with `cargo test --test reply`.

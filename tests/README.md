# Answer fixtures

`fixtures/noul.json`, `fixtures/choice.json`, and `fixtures/score.json` each contain
an answer of the corresponding type. These examples are based on the
[official SDK's response types](https://github.com/typesafe-ai/typesafe-sdk-js/blob/main/src/types.ts),
not captured API responses. The tests need no API key or network access.

Question IDs come from a global counter and aren't known in advance.
`common::reply_to` wraps the fixture's answer in a `Reply` using the ID of
the question you pass in:

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

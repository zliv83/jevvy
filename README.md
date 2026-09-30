# Jevvy

A Rust SDK for [TypeSafe AI](https://docs.typesafe.ai)'s Jev API.

You ask Jev questions about one piece of state, such as a news article, a support
ticket, or a chat log. Jev replies with one typed answer per question.

## Status

Alpha. This rebuild is published as `jevvy 0.2.0-alpha.1`. The builder interface
below works and has made live calls, but the API is still changing. A derive
interface is planned and does not exist yet. `0.1.0-alpha.1` was the previous
implementation, with a different API.

## Quickstart

```sh
cargo add jevvy@0.2.0-alpha.1
cargo add tokio --features macros,rt-multi-thread
```

Set `TYPESAFE_API_KEY`, then:

```rust
use jevvy::{Jev, JevvyError, NoulQuestion, Questions};

#[tokio::main]
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

  let relevance = reply.answer(&about_apple)?; // &NoulAnswer
  println!("probability of yes: {}", relevance.confidence().value());

  Ok(())
}
```

Jevvy is async. The example uses Tokio (`macros` and `rt-multi-thread`). From a
clone of this repository, run the same flow with `cargo run --example ask_jev`.

## What's here

| You ask          | Built with                                                           | Jev answers    | Read with                                                    |
|------------------|----------------------------------------------------------------------|----------------|--------------------------------------------------------------|
| `NoulQuestion`   | `.instructions()`, optional `.criteria(yes, no)`                     | `NoulAnswer`   | `.confidence()`, the probability of yes                      |
| `ChoiceQuestion` | `.instructions()`, 1–255 × `.option()` / `.option_without_description()` | `ChoiceAnswer` | `.choice()`, `.probabilities()`, `.confidence()`             |
| `ScoreQuestion`  | `.instructions()`, 2–10 × `.level()`, lowest level first             | `ScoreAnswer`  | `.score()`, `.legend()`, `.probabilities()`, `.confidence()` |

- `Questions::builder().state(...).question(&q).build()` groups any mix of
  question kinds about one state. Keep your question values: `reply.answer(&q)`
  uses them to find their answers, so there are no string keys.
- `Reply` also exposes `.model()` and `.usage()`.
- Values are exactly what Jev returned, never rounded or clamped. A Score can land
  between levels (for example `1.6`). A Noul near 0 is a confident no, while a low
  Choice/Score confidence means the probabilities are spread out.
- Every failure is a `JevvyError`: HTTP, a non-2xx status from Jev, decoding, a
  missing API key, builder validation, or an answer lookup (`MissingAnswer`,
  `WrongAnswerKind`).

## Examples

- `cargo run --example inspect_questions` prints the JSON Jevvy sends. It makes
  no API call.
- `cargo run --example ask_jev` asks Jev one live question. It needs
  `TYPESAFE_API_KEY`.

# Jevvy

A Rust SDK for [TypeSafe AI](https://docs.typesafe.ai)'s Jev API.

Ask Jev questions about a news article, a support ticket, a chat log, or other
data. That shared input is the state. Jev returns a typed answer for each question.

## Status

This rebuild is published as `jevvy 0.2.0-alpha.1`. The builder interface below
has been tested with live calls, but the API is still changing. A derive interface
is planned. The previous release, `0.1.0-alpha.1`, had a different API.

## Quickstart

```sh
cargo add jevvy@0.2.0-alpha.1
cargo add tokio --features macros,rt-multi-thread
```

Set the `TYPESAFE_API_KEY` environment variable, then ask a question:

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

Jevvy is async. This example uses Tokio with the `macros` and `rt-multi-thread`
features. To try it from a clone of this repository, run
`cargo run --example ask_jev`.

### Loading the key from `.env`

The optional `dotenvy` feature adds `Jev::from_dotenv()`:

```sh
cargo add jevvy@0.2.0-alpha.1 --features dotenvy
```

```rust
let jev = Jev::from_dotenv()?;
```

It finds a `.env` file in the current directory or a parent directory and loads
its variables into the environment. It then reads `TYPESAFE_API_KEY`, just like
`Jev::from_env()`. Variables already in the environment take precedence.

A missing or unreadable `.env` file returns `JevvyError::Dotenv`. A missing key
returns `JevvyError::MissingApiKey`. Add `.env` to your `.gitignore` to keep the
key out of your repository.

## What's here

| You ask          | Built with                                                           | Jev answers    | Read with                                                    |
|------------------|----------------------------------------------------------------------|----------------|--------------------------------------------------------------|
| `NoulQuestion`   | `.instructions()`, optional `.criteria(yes, no)`                     | `NoulAnswer`   | `.confidence()`, the probability of yes                      |
| `ChoiceQuestion` | `.instructions()`, 1–255 × `.option()` / `.option_without_description()` | `ChoiceAnswer` | `.choice()`, `.probabilities()`, `.confidence()`             |
| `ScoreQuestion`  | `.instructions()`, 2–10 × `.level()`, lowest level first             | `ScoreAnswer`  | `.score()`, `.legend()`, `.probabilities()`, `.confidence()` |

- `Questions::builder().state(...).question(&q).build()` combines questions
  about the same state in one request. You can mix question kinds. Keep each
  question to look up its answer with `reply.answer(&q)`; you don't need to
  manage string keys.
- `Reply` also exposes `.model()` and `.usage()`.
- Jevvy returns values exactly as Jev sent them, without rounding or clamping.
  A Score can fall between levels, such as `1.6`. A Noul value near 0 means a
  confident no. Low Choice or Score confidence means the probabilities are
  spread out.
- `JevvyError` covers HTTP failures, non-2xx responses from Jev, decoding,
  missing API keys, builder validation, and answer lookups (`MissingAnswer`
  or `WrongAnswerKind`).

## Examples

- `cargo run --example inspect_questions` prints the JSON Jevvy sends. It makes
  no API call.
- `cargo run --example ask_jev` asks Jev one live question. It needs
  `TYPESAFE_API_KEY`.

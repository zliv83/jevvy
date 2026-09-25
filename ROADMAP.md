# jevvy roadmap

A friendly, typed Rust client for the [TypeSafe AI System One API](https://docs.typesafe.ai/api).

**Where we are:** Stages 1–3 are done. **Next up: Stage 4, traits written by hand.**

_Last updated: 2026-09-25_

---

## Stages

| # | Stage | Status |
|---|-------|--------|
| 1 | Types | ✅ Done |
| 2 | Client core | ✅ Done |
| 3 | Builder and accessors | ✅ Done |
| 4 | Traits, written by hand | ⏭️ **Next** |
| 5 | Derive macros | ⬜ |
| 6 | Polish and publish | ⬜ |

---

## What works today

```rust
let jevvy = Jevvy::from_env()?;

let res = jevvy
  .evaluate("Help! My payouts have been failing for 3 days")
  .noul("is_urgent", "Does this convey urgency?")
  .choice("team", "Which team should handle this?", [
    ("billing", "Payments, invoicing, refunds"),
    ("technical", "Bugs, outages, integrations"),
  ])
  .score("mood", "How frustrated is the customer?", ["Calm", "Frustrated", "Very angry"])
  .send()
  .await?;

let urgent = res.noul("is_urgent")?;          // f64
let team = res.choice("team")?;               // &ChoiceAnswer
let mood = res.score("mood")?.score;          // f64
```

Last smoke test output:

```
urgent: 0.95
team: billing (72% sure)
mood: 1.04
```

---

## File map

```
src/
  lib.rs            pub mod builder, client, types
  client.rs         Jevvy: new, from_env, setters, execute (retry loop), evaluate
  builder.rs        RequestBuilder<'a>: noul, choice, score, question, model, send
  types.rs          re-exports entry::*
  types/
    entry.rs        Entry (untagged: Text | Object | Array) + From impls
    request.rs      Request { state: Entry, model, questions }
    questions.rs    Question (Noul | Choice | Score), constructors, NoulCriteria, ChoiceOption
    answers.rs      Answer (Noul | Choice | Score) wrapping NoulAnswer / ChoiceAnswer / ScoreAnswer
    response.rs     Response + getters (answer, noul, choice, score), Usage
    error.rs        JevvyError + handle_response
examples/
  smoke.rs          builder route, prints typed answers
  manual.rs         hand-built Request sent with execute()
```

---

## Stage 1: Types ✅

- `IndexMap` everywhere, so order is preserved.
- `Entry` is `#[serde(untagged)]`. Without it, serde wrote `{"Text": "..."}` instead of a plain string.
- `From` impls on `Entry` for `&str`, `String`, `&String`, and `serde_json::Value`. `&String` needs its own impl, because generic functions don't coerce it to `&str`.
- `instructions` is `Entry`, not `Option<Entry>`, because the API requires it.
- `Answer` uses newtype variants (`Noul(NoulAnswer)` and so on), so getters can hand out `&ChoiceAnswer`.
- `Usage` tokens are `u32`.

## Stage 2: Client core ✅

- `Jevvy::new(key)` plus chainable setters: `base_url`, `model`, `max_retries`.
- `from_env()` reads `TYPESAFE_API_KEY` (required), `TYPESAFE_BASE_URL`, `TYPESAFE_BASE_MODEL`, and `TYPESAFE_MAX_RETRIES`. A missing key gives `JevvyError::MissingApiKey`.
- `execute(&Request)` POSTs to `{base_url}/v1/systemone`, trimming any trailing `/`.
- Retries on 429 and 529 with backoff of ½s, 1s, 2s, 4s, then 8s max, up to `max_retries` (default 5). The status is checked *before* `handle_response`, so it doesn't read the body of a response it will throw away.
- `handle_response` maps 401, 422, 429, and 529 to named errors, and anything else to `Api { status, message }`.

## Stage 3: Builder and accessors ✅

- `jevvy.evaluate(state)` returns a `RequestBuilder<'_>`, the "shopping cart." It borrows the client and fills in the client's default model.
- Chain methods take `self` and return `Self`. The struct is `#[must_use]`, so a forgotten `.send()` gives a warning.
- `.question(key, Question)` is the escape hatch for anything the nice methods don't cover yet.
- `.send(self)` consumes the builder, so a cart can't be sent twice, and calls `execute`.
- `ChoiceOption` accepts a name only (`&str` or `String`) or a `(name, description)` tuple. No description serializes as `null`.
- Getters: `res.noul(key) -> f64`, `res.choice(key) -> &ChoiceAnswer`, `res.score(key) -> &ScoreAnswer`, and `res.answer(key) -> &Answer`.
- New errors: `MissingAnswer(key)` and `WrongAnswerType { key, expected, found }`.

---

## Stage 4: Traits, written by hand ⏭️ NEXT

**Goal:** swap string keys for real Rust types. Write every impl by hand first. That becomes the blueprint the macros will generate in Stage 5.

**Steps:**

1. Add generic answer wrappers that keep confidence, so users can match on confidence ranges:
   ```rust
   pub struct Choice<T> {
     pub value: T,
     pub confidence: f64,
     pub probabilities: IndexMap<T, f64>,
   }

   pub struct Score<T> {
     pub level: T,        // nearest level, e.g. Severity::Annoying
     pub raw: f64,        // the actual value, e.g. 1.05
     pub confidence: f64,
     pub probabilities: IndexMap<T, f64>,
   }
   ```
   Noul stays a plain `f64`. An optional helper: `choice.confident(0.9) -> Option<&T>`.
2. Define three traits:
   - `Options`: enum variants ↔ Choice criteria names and descriptions, and back again. Needs `Hash + Eq` for the `probabilities` keys.
   - `Levels`: ordered enum variants ↔ Score levels, plus picking the nearest level from `raw`.
   - `Rubric`: a struct → its questions, and a `Response` → that struct.
3. Hand-write all three for the issue-triage example below.
4. Add `jevvy.ask::<T: Rubric>(state).await -> Result<T, JevvyError>`.

**Target example (the issue-triage bot):**

```rust
struct Triage {
  is_bug: f64,                  // noul
  has_repro: f64,               // noul
  area: Choice<Area>,           // choice
  severity: Score<Severity>,    // score
}

enum Area { Networking, Serde, Docs }
enum Severity { Trivial, Annoying, Blocker }

let t: Triage = jevvy.ask(json!({ "title": title, "body": body })).await?;

match t.area.confidence {
  0.8.. => labels.push(t.area.value.label()),
  0.5..0.8 => labels.push("needs-triage"),
  _ => ping_maintainer(&issue).await?,
}
```

## Stage 5: Derive macros ⬜

- Turn the project into a workspace with a `jevvy-derive` proc-macro crate, using `syn` and `quote`.
- Build the macros in order of difficulty: `#[derive(Options)]`, then `#[derive(Levels)]`, then `#[derive(Rubric)]`.
- Check each macro's output against the hand-written impl from Stage 4.
- Test compile errors with `trybuild`.

**Target:**

```rust
#[derive(Rubric)]
struct Triage {
  #[noul("Is this a bug report, not a question or feature request?")]
  is_bug: f64,
  #[choice("Which part of the project does `title` concern?")]
  area: Choice<Area>,
  #[score("How badly does this affect users?")]
  severity: Score<Severity>,
}

#[derive(Options)]
enum Area {
  #[describe("HTTP client, retries, timeouts")] Networking,
  #[describe("Request/response types, JSON shape")] Serde,
  #[describe("README, examples, docs.rs")] Docs,
}
```

## Stage 6: Polish and publish ⬜

- Add `examples/triage.rs`, the full issue-triage bot.
- Add a mock transport so tests don't hit the network.
- Write docs with runnable examples, then publish to crates.io.

---

## Parked ideas

- Honor the `Retry-After` header, and add jitter to the backoff.
- A nicer builder method for Noul criteria (what "yes" and "no" mean).
- Client-side validation: Score needs 2–10 levels, Choice allows at most 255 options.
- Unit tests for `Entry` serialization and the `From` impls.

---

## How we work

- I type the code myself, so show code in chat with comments. Only edit files directly when I ask.
- Give an overview first, then go step by step.
- Keep explanations in simple English, with analogies and diagrams.
- Save issues outside the current file for a checkpoint instead of raising them mid-step.
- Style: 2-space indent, doc comments on public items.

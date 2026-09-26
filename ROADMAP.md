# jevvy roadmap

A friendly, typed Rust client for the [TypeSafe AI System One API](https://docs.typesafe.ai/api).

**Where we are:** Stages 1–4 are done. `jevvy.ask::<Triage>(issue)` makes a real API call and returns a typed struct. **Next up: Stage 5, derive macros**, starting with the workspace and `#[derive(Options)]`.

_Last updated: 2026-09-26_

---

## Stages

| # | Stage | Status |
|---|-------|--------|
| 1 | Types | ✅ Done |
| 2 | Client core | ✅ Done |
| 3 | Builder and accessors | ✅ Done |
| 4 | Traits, written by hand | ✅ Done |
| 5 | Derive macros | ⏭️ **Next** |
| 6 | Polish and publish | ⬜ |

---

## What works today

**The typed route (Stage 4).** Implement three traits once, then ask:

```rust
let jevvy = Jevvy::from_env()?;
let t: Triage = jevvy.ask(json!({ "title": title, "body": body })).await?;

t.is_bug            // f64
t.area.value        // Area::Networking
t.area.confidence   // f64
t.severity.level    // Severity::Blocker
t.severity.raw      // f64
```

Last real run (`cargo run --example triage`, an issue about jevvy's missing timeout):

```
is_bug: 0.94, has_repro: 0.96
area: Networking (100% sure)
severity: Blocker (raw 1.96)
labels: ["bug", "networking", "p0"]
```

**The quick route (Stage 3).** A builder with string keys, for one-offs:

```rust
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

let urgent = res.noul("is_urgent")?;   // f64
let team = res.choice("team")?;        // &ChoiceAnswer
let mood = res.score("mood")?.score;   // f64
```

---

## File map

```
src/
  lib.rs            pub mod builder, client, traits, types
  client.rs         Jevvy: new, from_env, setters, execute (retry loop), evaluate, ask
  builder.rs        RequestBuilder<'a>: noul, choice, score, question, model, send
  traits.rs         Options, Levels (each with a question() helper), Rubric
  types.rs          re-exports entry::*
  types/
    entry.rs        Entry (untagged: Text | Object | Array) + From impls
    request.rs      Request { state: Entry, model, questions }
    questions.rs    Question (Noul | Choice | Score), constructors, NoulCriteria, ChoiceOption
    answers.rs      Answer (Noul | Choice | Score) wrapping NoulAnswer / ChoiceAnswer / ScoreAnswer
    response.rs     Response + getters (answer, noul, choice, score), Usage
    error.rs        JevvyError + handle_response
    typed.rs        Choice<T>, Score<T>, TryFrom<&ChoiceAnswer>, TryFrom<&ScoreAnswer>
examples/
  smoke.rs          builder route, prints typed answers
  manual.rs         hand-built Request sent with execute()
  triage.rs         the issue-triage bot: Area, Severity, Triage, labels_for, a real ask() call
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

## Stage 4: Traits, written by hand ✅

**Goal:** swap string keys for real Rust types. Every impl was written by hand, so it can serve as the blueprint the Stage 5 macros will generate.

**The whole shape:** `Rubric` is the whole form, `Options` is the list of boxes in a multiple-choice question, `Levels` is a rating line from low to high.

```
 struct Triage: Rubric ──questions()──► Request ──► API
   is_bug:   f64                                     │
   area:     Choice<Area>     ◄── Options            │
   severity: Score<Severity>  ◄── Levels             ▼
 Triage ◄────────── from_response() ◄──────── Response
```

**What got built:**

1. **Answer wrappers** (`types/typed.rs`). `Choice<T> { value, confidence, probabilities: IndexMap<T, f64> }` and `Score<T> { level, raw, confidence, probabilities }`. Noul stays a plain `f64`. Helper: `choice.confident(min) -> Option<&T>`.
2. **`Options`** (`traits.rs`): `Copy + Eq + Hash + 'static`.
   - Required: `const ALL: &'static [Self]` and `name()`. Defaults: `describe()` (returns `None`), `from_name()`, and `question(instructions)`, which builds the whole Choice question.
   - `TryFrom<&ChoiceAnswer> for Choice<T>`, the "bouncer": every name must be in `ALL`, or it returns `UnknownOption(name)`.
3. **`Levels`** (`traits.rs`): same bounds.
   - Required: `ALL` (lowest first, so the order *is* the scale) and `describe()`, the text the model reads. Defaults: `from_index()`, `question()`, and `nearest(raw)`, which rounds, clamps to the scale, and rounds ties up (0.5 → position 1).
   - API facts: `probabilities` and `legend` are keyed by position (`"0"`, `"1"`, `"2"`), and `score` is their weighted average (0×0.00 + 1×0.95 + 2×0.05 = 1.05).
   - `TryFrom<&ScoreAnswer> for Score<T>`. `level_at` does two hops: `"1"` → `1` (parse) → variant (`from_index`). A bad key gives `UnknownLevel(key)`. `legend` is ignored, since the enum knows its own levels.
4. **`Rubric`**: `pub trait Rubric: Sized` with `questions() -> Questions` and `from_response(&Response) -> Result<Self, JevvyError>`. `Sized` is needed to return `Result<Self, _>`, the same bound `FromStr` and `Default` carry.
5. **`impl Rubric for Triage`** (`examples/triage.rs`). `Questions::from([(key, question), ...])` on one side and `res.choice("area")?.try_into()?` on the other. Every key string is typed **twice**, and a typo compiles but fails at runtime with `MissingAnswer`. That is exactly the pain `#[derive(Rubric)]` removes.
6. **`jevvy.ask::<T: Rubric>(state)`** (`client.rs`): `T::questions()` → `Request` → `execute()` → `T::from_response()`. `T` comes from a type annotation (`let t: Triage = ...`) or the turbofish (`ask::<Triage>`).

**Lessons worth keeping:**

- `'static` on the trait is needed because `&'static [Self]` promises the list lives forever, so every `Self` in it must too.
- `ALL` is a slice baked into the program at compile time. It isn't a `Vec`, and it never allocates.
- There are two roads between an enum and the API. Going out, `describe()`/`name()` turn a variant into text. Coming back, `ALL` + `from_index`/`from_name` turn text into a variant. The user's own `match` comes after both.
- `.map(...).collect::<Result<_, _>>()` is all or nothing, like an egg carton: one cracked egg sends the whole carton back. Errors never go *into* the map.
- `Result::map` only runs on `Ok`, and an `Err` passes straight through.
- `nearest` can pick a level the model never chose (a 50/50 Trivial/Blocker split gives raw 1.0, which is Annoying). That's why `raw` and `probabilities` are kept.
- `Vec::new` without `()` is a *fn item*, the function itself, not a Vec. It has no `.push()` and no `Debug`.
- Float ranges work as match patterns: `0.8.. =>`, `0.5..0.8 =>`.

## Stage 5: Derive macros ⏭️ NEXT

**What each macro must generate** (compare against the Stage 4 hand-written impls):

| Macro | Reads | Writes |
|-------|-------|--------|
| `#[derive(Options)]` | enum variants + `#[describe]` | `ALL`, `name()` (snake_case of the variant), `describe()` |
| `#[derive(Levels)]` | enum variants in order + `#[describe]` | `ALL`, `describe()` |
| `#[derive(Rubric)]` | fields + `#[noul]`/`#[choice]`/`#[score]` | `questions()` and `from_response()`, both keyed by the field name |

The user still derives `Clone, Copy, PartialEq, Eq, Hash` on the enums, because the traits require them.


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

- Polish `examples/triage.rs`. It already makes a real `ask()` call. Once the macros exist, switch it to the derives.
- ⚠️ `.env` (with the API key) was in the first commit. It's untracked and ignored now, but it's still in git history. Before adding a remote or publishing, rotate the key or rewrite history.
- Add a mock transport so tests don't hit the network.
- Write docs with runnable examples, then publish to crates.io.

---

## Parked ideas

- Honor the `Retry-After` header, and add jitter to the backoff.
- A nicer builder method for Noul criteria (what "yes" and "no" mean).
- Client-side validation: Score needs 2–10 levels, Choice allows at most 255 options.
- Unit tests for `Entry` serialization and the `From` impls.
- **HTTP timeout.** `reqwest::Client::new()` has none, so a silent server hangs `ask()` forever. jevvy triaged this bug against itself and rated it p0 😄. Use `Client::builder().timeout(...)`, plus a `timeout` setter.
- `UnknownOption` / `UnknownLevel` could name the question key, e.g. "`area`: unknown option `pizza`".
- Leftover nits: the 422 error message still says "Unauthorized"; `client.rs` typos ("hand-build", "methiods", "thats"); the `traits.rs` comment "od sources" should read "odd scores"; the issue text in `triage.rs` says "serer tha acctpts".
- `.git/stale-index.lock-from-claude` is a harmless leftover and safe to delete.

---

## How we work

- I type the code myself, so show code in chat with comments. Only edit files directly when I ask.
- Give an overview first, then go step by step.
- Keep explanations in simple English, with analogies and diagrams.
- Save issues outside the current file for a checkpoint instead of raising them mid-step.
- Style: 2-space indent, doc comments on public items.
- Claude doesn't run git commands on my machine. (A `git status` once left a stale `index.lock` behind.)

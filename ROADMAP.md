# jevvy roadmap

A typed Rust client for the [TypeSafe AI System One API](https://docs.typesafe.ai/api), rebuilt around the idea in `VISION.md`: software that notices, moves only when something changes, walks a book that code wrote, admits doubt, and keeps receipts.

The first build, its roadmap, the migration map (`MIGRATION.md`) and the review that started the rebuild (`DIVERGENCES.md`) live in git history. The first build's lessons are kept at the bottom of this file.

**Where we are:** Stages 1, 2, 3 and 5 are done. **Next up: Stage 6, receipts and replay.** Stage 4 (derive) can land any time; nothing depends on it.

**Open before Stage 6:** delete the leftover `wrong_type` function (and the imports it used) from `src/types/jevvy_response.rs`; it moved to `sheet.rs` and is now dead code.

_Last updated: 2026-09-27_

---

## The system in one picture

Five sentences from the vision become five pieces of the crate, stacked on a wire layer that mirrors the API exactly.

```
 9  Polish     docs, tests, clippy, crates.io                                      ⬜
 8  Walk       Page, Turn, Trail            "it walks the book"                     ⬜
 7  Watch      sensors, change loop, diff   "it notices, and moves only on change"  ⬜
 6  Ledger     Receipt, Ledger, Replay      "it keeps receipts, and can replay"     ⏭️
 5  Gates      Gate, NoulGate, Verdict      "it knows what it doesn't know"         ✅
 4  Derive     #[derive(Form / Options / Levels)]                                   ⬜
 3  Batch      many forms, one request      "one moment, one call"                  ✅
 2  Forms      typed questions, typed answers, sheets                               ✅
 1  Wire       Entry, Question, Answer, JevvyRequest, JevvyResponse, Transport      ✅
```

Read it bottom-up as "what talks to the API" (1), "how you write questions in Rust" (2–4), and "how a running program uses the answers" (5–8).

The mental model that holds it together:

- **State** is the material. A snapshot of whatever your program is looking at right now: a ticket, a conversation so far, a record next to a policy. Code builds it, code filters it, and questions point into it by name.
- **A Form** is a sheet of questions with a typed answer sheet on the back. Some forms are typed out once (a triage form). Some are stamped out of data (one form per candidate record). Either way, a form *contributes* questions to a request; it never owns the request.
- **A Batch** is one request: one state, every form and loose question that applies to this moment. The reply is read back through the same forms.
- **A Gate** turns an answer plus its confidence into one of three verdicts: act, confirm, escalate. Thresholds are yours.
- **A Receipt** is a request and its reply, kept. A **Ledger** is the append-only file of receipts. **Replay** serves a ledger back as if it were the API (tests without network) or runs new questions over old states (backtests).
- **A Watch** holds a state and a panel of forms. Change the state and `tick()`; it sends one batch only if something changed, and hands back the answers plus a diff against last time. Standing questions become sensors.
- **A Walk** is the book. Code writes **Pages**; each page asks a batch and turns to the next page, finishes, or admits it is unsure. Jev picks; code turns the page. The **Trail** is the receipts of one reading.

---

## Vocabulary

The docs' words win wherever Rust allows it. Anyone moving between docs.typesafe.ai and the crate should never have to translate.

| Thing | On the wire (docs) | In the crate |
|---|---|---|
| The material | `state` | `state: Entry` |
| One judgment | `question` with `type`, `instructions`, `criteria` | `Question::{Noul, Choice, Score}` |
| The key you choose | question id | a form's short key, prefixed by the sheet (`triage.team`, `dup.18.same_person`) |
| Choice answers | `choice`, `probabilities`, `confidence` | `Choice<T> { choice, probabilities, confidence }` |
| Score answers | `score`, `legend`, `probabilities`, `confidence` | `Score<T> { score, probabilities, confidence }` plus `nearest()`, `most_likely()`, `normalized()`, `at_least()` |
| Noul answer | `noul` | `f64` |
| Noul criteria | `true`, `false` | `when_true`, `when_false` (serialized as `true` / `false`); `noul_with(key, instructions, when_true, when_false)` |
| Choice options | `criteria` map | an `Options` enum: `ALL`, `name()`, `describe() -> Option<Entry>` |
| Score levels | `criteria` array, lowest first | a `Levels` enum: `ALL`, `describe() -> Entry`, `index()` |
| Any description | string, object, array, or null | `Entry`, or `Option<Entry>` where null is allowed |
| A set of questions | (no docs word) | `Form`: `ask` onto a `QuestionSheet`, `read` from an `AnswerSheet` |
| One request | request body | `Batch` (building) / `JevvyRequest` (built) |
| One reply | response body | `JevvyResponse`, read through `JevvyResponse::sheet()` |
| The three paths | act, confirm, escalate | `Gate`, `NoulGate`, `Verdict<T>` |

The word "rubric" is retired. The docs use it for one question's criteria; we don't need it.

---

## Design rules

Each rule fixes a way the first build drifted from the TypeSafe docs; the *why* says how.

1. **Every description slot is an `Entry`.** Instructions, option descriptions, level descriptions, Noul sides. Strings stay easy (`"..."` converts), objects are welcome (`json!({...})`), and the reply's `legend` is read as `Entry` too. *Why:* the docs' best questions are objects with labelled parts (`what`, `not_for`, `examples`); one-line strings couldn't hold them, and a structured Score level couldn't be read back.
2. **A form contributes; a batch owns.** `Form::ask` writes into a sheet it was handed. Two forms, twenty forms, a loop of forms: one request. *Why:* every question about one moment belongs in one request. When a type owned its request, reuse meant one giant struct or extra round trips.
3. **Forms can carry data.** A form is a value. A unit struct is a constant form; a struct with fields is a form built from a record. Both implement the same trait. *Why:* the docs stamp questions out of data at run time (one per candidate record); a form fixed at compile time couldn't.
4. **A Score is a position.** The typed wrapper keeps the docs' `score` as its first field. Rounding is a method you call on purpose, next to `most_likely()` and the probabilities. *Why:* rounding first hides splits. A 50/50 Trivial/Blocker answer rounds to Annoying, a level the model never chose.
5. **Keys are prefixed, never retyped.** A form's field names are its local keys. The sheet adds a prefix (`triage.area`, `dup.18.same_person`). The derive macro writes both the asking and the reading side, so a key is spelled once. *Why:* a key typed twice compiles with a typo in it and fails at run time with `MissingAnswer`.
6. **Questions point into the state.** The form's instructions name the parts they judge with backticked paths, and the example state for a form lives next to it in a doc comment, so the two are written together. *Why:* hand a form a differently shaped state and its paths point at nothing, and the compiler can't tell.
7. **Keep the receipt.** Nothing in the crate discards `usage` or the model version. Every send can be recorded. *Why:* receipts are what replay, tests and backtests run on. The first build threw them away.
8. **One moment, one call.** The crate never sends twice for the same state on its own. A second request happens because a page turned or the state changed. *Why:* Jev gives the same answer for the same moment, so asking twice costs money and learns nothing.

---

## A day in the life

The finished thing, as it would read in a support-desk program. Every name here is built in a stage below.

```rust
// Stage 2–4: forms, typed once
#[derive(Form)]
struct Triage {
  #[choice("Which team should handle `ticket.message`?")]
  team: Choice<Team>,
  #[noul(question = "Does `ticket.message` ask for money back or a credit?",
         when_true = "Directly asks for a refund or credit",
         when_false = "A complaint or question without a requested remedy")]
  refund_requested: f64,                    // speculative: read only on the billing path
  #[score("How frustrated does the customer appear in `ticket.message`?")]
  frustration: Score<Mood>,
}

// Stage 2–3: a form stamped out of data
struct SamePerson<'a> { record: &'a Candidate }

// Stage 5: gates, thresholds in code
const ROUTE: Gate = Gate { act: 0.75, confirm: 0.5 };

// Stage 7: the inbox is a Watch. New message -> tick -> answers + diff.
let mut inbox = jevvy.watch(conversation, TriageForm);
inbox.update(|c| c.messages.push(incoming));
if let Some(reading) = inbox.tick().await? {
  let t = reading.answers;
  match ROUTE.judge(&t.team) {
    Verdict::Act(Team::Billing) => billing(ticket, t.refund_requested >= 0.7),
    Verdict::Act(team)          => route(ticket, team),
    Verdict::Confirm(team)      => suggest(ticket, team),
    Verdict::Escalate           => human(ticket),
  }
  for change in reading.changes.iter().filter(|c| c.magnitude() > 0.3) {
    log_alarm(change);                        // "frustration rose 1.1 -> 1.9"
  }
}

// Stage 8: a book for the hard cases
let trail = jevvy.walk(state, Box::new(DepartmentPage)).await?;

// Stage 6: last year's tickets, this year's rule
let ledger = Ledger::open("receipts.jsonl")?;
for receipt in ledger.iter() {
  let receipt = receipt?;
  let then = receipt.response.sheet().form("triage", &TriageForm)?;
  let now  = jevvy.fill(receipt.request.state.clone(), &NewRuleForm).await?;
  compare(then, now);
}
```

---

## Stages

Each unfinished stage lists its goal, the steps in the order to type them, a target snippet, and what "done" looks like. Every step is written by hand first; the macros in Stage 4 copy what the hand wrote. The examples named in Stages 6–8 are ideas for the one new example (see How we work), not per-stage checkpoints.

### Stage 1: Wire ✅

**Goal:** the crate speaks the API's exact dialect, with the docs' names, structure allowed everywhere the docs allow it, and a transport you can swap.

**What landed**

- `Entry` (`types/entry.rs`): `Text | Object | Array`, untagged. `Object` holds a `serde_json::Map`, which is an `IndexMap` underneath (`preserve_order`). `From` for `&str`, `String`, `&String`, `Value`, `bool`, `f64` (numbers and bools become text, because the API reads text).
- `Question` (`types/questions.rs`): `noul`, `noul_with`, `choice`, `score`, and `validate(key)`: 1–255 options, 2–10 levels, from the constants `MAX_OPTIONS`, `MIN_LEVELS`, `MAX_LEVELS`. `NoulCriteria { when_true, when_false }`. Map names `Questions` and `ChoiceCriteria`.
- Answers (`types/answers.rs`): `Answer::{Noul, Choice, Score}`, `legend: Legend` (`IndexMap<String, Entry>`), `Probabilities<K = String>`, and `Answers`, a `#[serde(transparent)]` struct with `get`, `iter`, `len`, `is_empty`.
- `JevvyRequest` and `JevvyResponse` (`types/jevvy_request.rs`, `types/jevvy_response.rs`), named so they never read as reqwest's types. `JevvyResponse` keeps `model` and `usage`.
- `JevvyError` (`types/jevvy_error.rs`), `#[non_exhaustive]`, in four families: setup (`MissingApiKey`); the call (`Reqwest`, `Timeout { after }`, `Unauthorized`, `UnprocessableEntity { field, message }`, `RateLimited`, `Overloaded`, `Api`); refused before sending (`BadOptionCount`, `BadLevelCount`); reading the reply (`MissingAnswer`, `WrongAnswerType`, `UnknownOption { key, found }`, `UnknownLevel { key, found }`).
- `Transport` (`transport.rs`): `fn send(&self, &JevvyRequest) -> impl Future<Output = Result<JevvyResponse, JevvyError>> + Send`. `Http` retries 429/529, honoring `Retry-After` (capped at 60 s) or else waiting ½ s doubling to 8 s; times out each try after 30 s by default; never retries a timeout. `check_status` (private) maps status codes to errors.
- `Jevvy<T = Http>` (`client.rs`): `new`, `from_env`, `base_url`, `max_retries`, `timeout` on `Jevvy<Http>`; `with_transport`, `model`, `send` on any `Jevvy<T: Transport>`. `send` validates every question, then hands the request to the transport.

**Changed from the plan:** `BadOptionCount` instead of `TooManyOptions`, since it also catches zero. `validate` takes the key so the error names the question. `impl Future + Send` instead of `async fn` in the trait. No bound on the `Jevvy` struct itself; bounds sit on the `impl` blocks. `Request`, `Response` and `JevvyError::Request` were renamed `JevvyRequest`, `JevvyResponse` and `JevvyError::Reqwest`.

**Still open:** the live checks. Nothing has hit the real API since the rebuild. A structured Score level round-trip, a 256-option Choice refused before sending, retry with `Retry-After`, and a timeout were all checked locally against fake transports and a fake server. The real run happens with Stage 6's recorded fixtures and the new example.

---

### Stage 2: Forms ✅

**Goal:** typed questions and typed answers, with the docs' names, and a `Form` trait that writes into a sheet it is handed rather than owning a request.

**The shape:**

```
                 QuestionSheet (prefix "triage")            AnswerSheet (prefix "triage")
 TriageForm ──ask()──► noul("is_bug", ..)  ──► "triage.is_bug"   ◄── read() ◄── Triage { is_bug, area, severity }
                       choice::<Area>("area", ..)                      choice::<Area>("area")?
                       score::<Severity>("severity", ..)               score::<Severity>("severity")?
```

**What landed**

- `traits.rs`: `Options::describe() -> Option<Entry>`, `Levels::describe() -> Entry`, `Levels::index()`, and `Form { type Answers; ask(&self, &mut QuestionSheet<'_>); read(&self, &AnswerSheet<'_>) }`. `Rubric` is gone.
- `typed.rs`: `Choice<T> { choice, probabilities, confidence }` with `from_answer(key, &ChoiceAnswer)` and `confident(min)` (the pick, if confidence is at least `min`). `Score<T> { score, probabilities, confidence }` with `from_answer(key, &ScoreAnswer)`, `nearest()`, `most_likely()` (ties go to the higher level), `normalized()`, `at_least(level)`.
- `sheet.rs`: `QuestionSheet` (`root`, `key`, `question`, `noul`, `noul_with`, `choice::<T>`, `score::<T>`, `form`, `each`; every writer returns `&mut Self` so calls chain) and `AnswerSheet` (`root`, `key`, `raw`, `noul`, `choice::<T>`, `score::<T>`, `form`, `each`). Keys join with a dot; an empty side means no dot. The old getters on the response moved here; `JevvyResponse::sheet()` gives the root.

**Changed from the plan:** the `TryFrom` bouncers became `from_answer(key, answer)`, because `TryFrom` takes one input and the key had nowhere to go. `noul_with` takes both sides as arguments rather than a `NoulCriteria`. `each` arrived here instead of Stage 3. `fill` moved to Stage 3, on top of `Batch`. No triage example; the examples were deleted.

---

### Stage 3: Batch ✅

**Goal:** one request assembled from any number of forms, repeated forms, and loose questions; the reply read back through the same forms.

**What landed**

- `batch.rs` (was `builder.rs`): `Batch<'a, T = Http>`, `#[must_use]`, made by `jevvy.ask(state)`. Every chain method writes through a root `QuestionSheet`: `question`, `noul`, `noul_with`, `choice::<O>`, `score::<L>`, `form(prefix, &form)`, `each(prefix, pairs)`, plus `model`, `request()` (look before you send) and `send()`.
- `Jevvy::fill(state, &form)`: `ask(state).form("", form).send()`, then `sheet().form("", form)`. It returns only the answers; Stage 6's `Recording` keeps the receipt.

**Target**

```rust
let res = jevvy
  .ask(json!({ "ticket": ticket, "customer": customer }))
  .form("triage", &TriageForm)
  .form("spam",   &SpamSignals)
  .each("dup", candidates.iter().map(|c| (c.id, SamePerson { record: c })))
  .noul("mentions_open_order", "Does `ticket.message` refer to any of `customer.open_orders`?")
  .send()
  .await?;

let sheet = res.sheet();
let triage = sheet.form("triage", &TriageForm)?;
let spam   = sheet.form("spam", &SpamSignals)?;
let dups   = sheet.each("dup", candidates.iter().map(|c| (c.id, SamePerson { record: c })))?;
```

**Changed from the plan:** the old untyped `choice(key, instructions, [names])` and `score(key, instructions, [levels])` are gone; options built at run time go through `.question(key, Question::choice(..))`. The method generics are `O` and `L` because `T` is the transport. No duplicates example; the same shape was checked locally (two `SamePerson` forms under `dup.18` and `dup.42`, one request, every key once).

---

### Stage 4: Derive ⬜

**Goal:** `#[derive(Form)]`, `#[derive(Options)]`, `#[derive(Levels)]`, generating exactly what a hand-written `Form`, `Options` or `Levels` impl says. Structured descriptions are first-class in the attributes.

**Steps**

1. Turn the project into a workspace. New crate `jevvy-derive` (proc-macro) with `syn`, `quote`, `proc-macro2`. Re-export the macros from `jevvy` behind a default `derive` feature.
2. `#[derive(Options)]` on an enum. Reads `#[name("...")]` (optional, default snake_case of the variant) and `#[describe(...)]` in two forms: a string literal, or key = value pairs (`what = "...", not_for = "...", examples = ["..", ".."]`), which become an `Entry::Object` with those keys. Writes `ALL`, `name()`, `describe()`.
3. `#[derive(Levels)]` on an enum. Same `describe` forms. Writes `ALL` in declaration order and `describe()`.
4. `#[derive(Form)]` on the *answers* struct. Reads `#[noul(...)]`, `#[choice(...)]`, `#[score(...)]` per field; each takes a string literal or key = value pairs for structured instructions, and `noul` also accepts `when_true = ...`, `when_false = ...`. Generates a unit struct named `<Name>Form` and `impl Form for <Name>Form { type Answers = <Name>; ... }` whose `ask` writes through the `QuestionSheet` and whose `read` reads through the `AnswerSheet`, keyed by the field names. The field types decide the reader: `f64`, `Choice<T>`, `Score<T>`.
5. `trybuild` tests: a `Choice<T>` field where `T` is not `Options`, a struct with an unannotated field, a `describe` with an unknown key. Each must fail with a message that names the field.
6. A test that compares a derived form's request JSON to a hand-written form's, byte for byte.

**Target**

```rust
#[derive(Clone, Copy, PartialEq, Eq, Hash, Options)]
enum Team {
  #[describe(what = "Charges, invoices, refunds, or subscriptions",
             not_for = "Order tracking or account access",
             examples = ["I was charged twice", "Where is my refund?"])]
  Billing,
  #[describe("Order status, delivery, cancellation, or returns")]
  Orders,
  #[name("none_of_the_above")] #[describe("None of the teams above fits")]
  Other,
}

#[derive(Form)]
struct Triage {
  #[choice("Which team should handle `ticket.message`?")]
  team: Choice<Team>,
  #[noul(question = "Does `ticket.message` ask for money back or a credit?",
         when_true = "Directly asks for a refund or credit",
         when_false = "A complaint or question without a requested remedy")]
  refund_requested: f64,
  #[score("How frustrated does the customer appear in `ticket.message`?")]
  frustration: Score<Mood>,
}
```

Forms built from data stay hand-written; the derive covers the constant case, which is most forms.

**Done when** the derived form produces the same JSON as the hand-written one and the `trybuild` errors read like advice.

---

### Stage 5: Gates ✅

**Goal:** the three paths from the docs (act, confirm, escalate) as a small typed vocabulary, with every threshold still in your code. This is the "unsure door".

**What landed**

- `gate.rs`: `Verdict<T> { Act(T), Confirm(T), Escalate }` with `is_sure()`, `gate_map()`, and `into_option()` (a value for `Act` and `Confirm`, `None` for `Escalate`).
- `Gate { act, confirm }` with `judge(&Choice<T>) -> Verdict<T>` and `judge_score(&Score<T>) -> Verdict<f64>`. Set `confirm` equal to `act` for no confirm band.
- `NoulGate { yes, no }` with `judge(p) -> Verdict<bool>`; the band between `no` and `yes` escalates.
- Public fields, so a gate is a `const` next to the code that uses it, named for the action it guards.

**Target**

```rust
const SHOW_BALANCE:     Gate = Gate { act: 0.60, confirm: 0.60 };
const APPROVE_TRANSFER: Gate = Gate { act: 0.85, confirm: 0.60 };

match intent.choice {
  Intent::CheckBalance    => match SHOW_BALANCE.judge(&intent)     { Verdict::Act(_) => show_balance(), _ => agent() },
  Intent::ApproveTransfer => match APPROVE_TRANSFER.judge(&intent) {
    Verdict::Act(_)     => approve(),
    Verdict::Confirm(_) => ask_user_to_confirm(),
    Verdict::Escalate   => agent(),
  },
  Intent::Other => agent(),
}
```

**Changed from the plan:** `Verdict<T>` instead of `Verdict<&T>`. Option enums are `Copy`, so `Verdict::Act(Team::Billing)` matches directly. `map` was renamed `gate_map` so it doesn't sit beside `Option::map` in autocomplete. No gates example; the same checks ran locally (at 0.7 confidence `SHOW_BALANCE` acts and `APPROVE_TRANSFER` confirms; 0.5 on a 0.8/0.2 `NoulGate` escalates).

---

### Stage 6: Ledger and replay ⏭️ NEXT

**Goal:** every send can be kept as a receipt; a ledger of receipts can stand in for the API (tests with no network) or be walked again with new questions (backtests). This is also where the first real tests arrive, and where the crate first talks to the real API since the rebuild.

**Steps**

1. `receipt.rs`, new. `Receipt { at_ms: u64, request: JevvyRequest, response: JevvyResponse }`, `Serialize + Deserialize`. `at_ms` is Unix milliseconds from `SystemTime`, so no new dependency.
2. `ledger.rs`, new. `Ledger` over a JSON Lines file: `open(path)` (creates if missing), `append(&Receipt)`, `iter() -> impl Iterator<Item = Result<Receipt>>`, `len()`.
3. `transport::Recording<T>`. Wraps any transport; on every successful send it appends a receipt to a ledger. `jevvy.recording("receipts.jsonl")?` returns a `Jevvy<Recording<Http>>`.
4. `transport::Replay`. Loads a ledger into a map keyed by the request serialized to a canonical JSON string. `send` returns the recorded response or `JevvyError::NotRecorded` (a new variant in `jevvy_error.rs`). `Jevvy::replay(path)`.
5. `tests/`. Record a small form once against the real API with `Recording` into `fixtures/*.jsonl` (this is the live check Stage 1 deferred: include a structured Score level), then tests that run the same form through `Replay` and assert on the typed answers. Same for a batch with `each`. Also unit tests for `Entry` serialization and the `From` impls.
6. A backtest (a test or part of the new example): open a ledger, and for each receipt send its *state* with a *new* form (a changed threshold, a reworded level), then print old verdict next to new verdict.

**Target**

```rust
let jevvy = Jevvy::from_env()?.recording("receipts.jsonl")?;   // every send is kept

// later, offline
let jevvy = Jevvy::replay("fixtures/triage.jsonl")?;
let t: Triage = jevvy.fill(issue, &TriageForm).await?;          // served from the file

// later still, a backtest
for r in Ledger::open("receipts.jsonl")?.iter() {
  let r = r?;
  let then = r.response.sheet().form("", &TriageForm)?;
  let now  = live.fill(r.request.state.clone(), &TriageV2Form).await?;
  println!("{:?} -> {:?}", ROUTE.judge(&then.team), ROUTE.judge(&now.team));
}
```

**Done when** `cargo test` passes with no network and no API key, and a backtest prints a before/after column for every receipt in a ledger.

---

### Stage 7: Watch ⬜

**Goal:** a state your program mutates, a panel of standing questions, and a `tick()` that sends one batch only when the state changed, returning the answers and a diff against the last reading. Standing questions become sensors. This is "it notices" and "it moves only on change".

**Steps**

1. `diff.rs`, new. `Change { key: String, kind: ChangeKind }` with `ChangeKind::{ Noul { from, to }, Choice { from, to }, Score { from, to }, Confidence { from, to }, Added, Removed }` and `magnitude() -> f64` (absolute delta for numbers, 1.0 for a changed choice, 0.0 otherwise). `diff(before: &Answers, after: &Answers) -> Vec<Change>`, keyed match.
2. `watch.rs`, new. `Watch<'a, T, S, F>` where `S: Serialize` and `F: Form`, made by `jevvy.watch(state, form)`. Holds `state`, `form`, `dirty: bool`, `last: Option<Receipt>`.
3. `update(&mut self, f: impl FnOnce(&mut S))` runs `f` and marks dirty. `set(S)` replaces. `state(&self) -> &S`.
4. `tick(&mut self) -> Result<Option<Reading<F::Answers>>>`. Not dirty: `Ok(None)`, no request. Dirty: serialize state to `Entry`, batch the form at the root, send, diff against `last`, store the receipt, clear dirty, return `Some(Reading { answers, changes, receipt })`. `force()` ticks even when clean.
5. `Reading::alarms(min_magnitude) -> impl Iterator<Item = &Change>`.
6. Example idea, `inbox`: a conversation struct as state. Push six messages one at a time from a script, tick after each, print the frustration score, the team, and every change above 0.3. Then push the same message twice and show the second tick returns `None`.

**Target**

```rust
let mut inbox = jevvy.watch(Conversation::new(customer), TriageForm);

for msg in incoming {
  inbox.update(|c| c.messages.push(msg));
  if let Some(r) = inbox.tick().await? {
    println!("frustration {:.2}  team {:?}", r.answers.frustration.score, r.answers.team.choice);
    for c in r.alarms(0.3) { println!("  alarm: {c}"); }
  }
}
```

**Done when** a watch sends one request per changed state, zero for an unchanged one, and reports a diff line when frustration climbs.

---

### Stage 8: Walk ⬜

**Goal:** the book. Code writes pages; each page asks a batch, then turns to another page, finishes, or admits it is unsure. Jev picks at every fork; code turns the page; the trail of receipts is the record. This generalizes the docs' taxonomy walk and skill-suggestion cookbook.

**Steps**

1. `walk.rs`, new.
   ```rust
   pub trait Page {
     type Outcome;
     fn ask(&self, sheet: &mut QuestionSheet<'_>);
     fn turn(&self, sheet: &AnswerSheet<'_>) -> Result<Turn<Self::Outcome>, JevvyError>;
   }
   pub enum Turn<O> {
     Next(Box<dyn Page<Outcome = O>>),
     NextWith { page: Box<dyn Page<Outcome = O>>, state: Entry },   // fetched more, new material
     Done(O),
     Unsure(String),
   }
   ```
2. `Trail<O> { receipts: Vec<Receipt>, end: End<O> }` with `End::{ Done(O), Unsure(String), TooLong }`.
3. `jevvy.walk(state, first: Box<dyn Page<Outcome = O>>) -> Result<Trail<O>>`. Loop: root sheet, `page.ask`, send, `page.turn`; on `Next` keep the state, on `NextWith` swap it, on `Done`/`Unsure` stop. `max_pages` defaults to 8 and is a setter. The cap is what keeps this a workflow and not an agent loop: pages are finite and code-authored.
4. Example idea, `taxonomy`: `DepartmentPage` asks one Choice over top-level departments, each option's description being the subtree (rule 1 makes that a `json!` object). `turn` gates on confidence: `Act` goes to `SubIssuePage { department }`, `Confirm` also goes on but marks the trail, `Escalate` returns `Unsure`. The leaf page returns `Done(path)`.
5. Print the trail: pages visited, the chosen option and confidence at each, and total tokens from the receipts.

**Target**

```rust
struct DepartmentPage;
impl Page for DepartmentPage {
  type Outcome = Vec<Team>;
  fn ask(&self, s: &mut QuestionSheet<'_>) { s.choice::<Department>("department", "Which department does `listing` belong to?"); }
  fn turn(&self, s: &AnswerSheet<'_>) -> Result<Turn<Vec<Team>>, JevvyError> {
    let d = s.choice::<Department>("department")?;
    Ok(match CLASSIFY.judge(&d) {
      Verdict::Act(dep) | Verdict::Confirm(dep) => Turn::Next(Box::new(SubIssuePage { department: dep })),
      Verdict::Escalate => Turn::Unsure(format!("department split: {:?}", d.probabilities)),
    })
  }
}

let trail = jevvy.walk(json!({ "listing": listing }), Box::new(DepartmentPage)).await?;
```

**Done when** a clear listing reaches a leaf in two requests, an ambiguous one stops with `Unsure`, and the trail lists both.

---

### Stage 9: Polish and publish ⬜

- `.env` is out of git history now. If the old history was ever pushed, rotate the key.
- The new example: one real program against the live API (`examples/smoke.rs` is its placeholder). It covers whatever the fixtures in Stage 6 didn't: a structured Score level round-trip, and a 256-option Choice refused before it leaves the machine.
- Turn clippy back up (pedantic) and clear it. `typed.rs` keeps its file-wide `#![allow(clippy::cast_precision_loss)]`.
- README: the vision in three paragraphs, then the day-in-the-life snippet, then one short example per layer.
- Doc comments with runnable examples on every public item; `cargo doc` clean; `cargo test` runs the doctests through `Replay` fixtures.
- Delete `.git/stale-index.lock-from-claude`, a harmless leftover from an old `git status`.
- `CHANGELOG.md`, license, `cargo publish --dry-run`, then publish `jevvy-derive` first and `jevvy` second.

---

## Parked ideas

- Jitter on the backoff.
- A beam-search helper for `walk`: keep the top K branches when a Choice splits.
- A `Diffable` derive so `Reading` can diff typed answers field by field, not just by key.
- A `Panel` that is several forms watched together (today: compose them into one form).
- Readings kept as a time series, so a Watch can answer "how did frustration move over the last hour".
- Ledger compaction and a size cap.
- A blocking client for scripts.

---

## How we work

- I type the code myself, so show code in chat with comments. Only edit files directly when I ask.
- Give an overview first, then go step by step. Keep explanations in simple English, with analogies and diagrams.
- **No throwaway code.** Go straight to the final shape. Reuse or move existing code instead of retyping it. Never write placeholder code just to keep something compiling for one step.
- A step's checkpoint is `cargo check`. Examples are deferred: one new example once the new layers are in, plus Stage 6's tests.
- Clippy stays relaxed until Stage 9.
- Save issues outside the current file for a checkpoint instead of raising them mid-step.
- Style: 2-space indent, doc comments on public items.
- When a stage lands, update its mark, the status line, and `_Last updated_` here.

---

## Lessons worth keeping

From the first build:

- `Entry` needs `#[serde(untagged)]`; without it serde writes `{"Text": "..."}` instead of a plain string.
- `&String` needs its own `From` impl, because generic functions don't coerce it to `&str`.
- `'static` on `Options` and `Levels` is needed because `&'static [Self]` promises the list lives forever, so every `Self` in it must too.
- `ALL` is a slice baked into the program at compile time. It isn't a `Vec`, and it never allocates.
- There are two roads between an enum and the API. Going out, `describe()`/`name()` turn a variant into text. Coming back, `ALL` + `from_index`/`from_name` turn text into a variant. Your own `match` comes after both.
- `.map(...).collect::<Result<_, _>>()` is all or nothing, like an egg carton: one cracked egg sends the whole carton back.
- `Result::map` only runs on `Ok`, and an `Err` passes straight through.
- `nearest` can pick a level the model never chose (a 50/50 Trivial/Blocker split averages to Annoying). That's why `score` is the headline and rounding is a method.
- `Vec::new` without `()` is a *fn item*, the function itself, not a Vec.
- Float ranges work as match patterns: `0.8.. =>`, `0.5..0.8 =>`.

From the rebuild:

- A trait isn't a type: `form: Form` doesn't compile. `<F: Form>` names the type, so a return type can say `F::Answers`.
- A method can't reuse its `impl`'s generic name. `Batch<T>` already uses `T` for the transport, so its methods use `O` and `L`.
- `&mut *self.questions` lends a borrow to a child sheet for a moment, then takes it back.
- `3.0_f64.to_string()` is `"3"`; `Display` drops a trailing `.0`.
- Zed's F2 (Rename Symbol) renames a symbol everywhere it's used, but not in comments.

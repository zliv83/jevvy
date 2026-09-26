# jevvy roadmap 2

A typed Rust client for the [TypeSafe AI System One API](https://docs.typesafe.ai/api), rebuilt around the idea in `VISION.md`: software that notices, moves only when something changes, walks a book that code wrote, admits doubt, and keeps receipts.

This roadmap supersedes `ROADMAP.md`, which stays as the record of Stages 1–4 of the first build. It starts fresh. Nothing here *requires* the old code, but most of the wire layer carries over; `MIGRATION.md` says exactly what to keep, rename, and drop. `DIVERGENCES.md` is the review that led here.

**Where we are:** the old Stage 4 is done. **Next up: Stage 1, the wire**, which is mostly a rename-and-fix pass over what exists.

_Last updated: 2026-09-26_

---

## The system in one picture

Five sentences from the vision become five pieces of the crate, stacked on a wire layer that mirrors the API exactly.

```
 9  Polish     docs, tests, crates.io
 8  Walk       Page, Turn, Trail            "it walks the book"
 7  Watch      sensors, change loop, diff   "it notices, and moves only on change"
 6  Ledger     Receipt, Ledger, Replay      "it keeps receipts, and can replay"
 5  Gates      Gate, Verdict                "it knows what it doesn't know"
 4  Derive     #[derive(Form / Options / Levels)]
 3  Batch      many forms, one request      "one moment, one call"
 2  Forms      typed questions, typed answers, sheets
 1  Wire       Entry, Question, Answer, Request, Response, Client, Transport
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
| The key you choose | question id | field name inside a form, prefixed by the batch |
| Choice answers | `choice`, `probabilities`, `confidence` | `Choice<T> { choice, probabilities, confidence }` |
| Score answers | `score`, `legend`, `probabilities`, `confidence` | `Score<T> { score, probabilities, confidence }` plus `nearest()`, `most_likely()`, `normalized()` |
| Noul answer | `noul` | `f64` |
| Noul criteria | `true`, `false` | `when_true`, `when_false` (serialized as `true` / `false`) |
| Choice options | `criteria` map | an `Options` enum: `ALL`, `name()`, `describe()` |
| Score levels | `criteria` array, lowest first | a `Levels` enum: `ALL`, `describe()` |
| Any description | string, object, array, or null | `Entry`, or `Option<Entry>` where null is allowed |
| A set of questions | (no docs word) | `Form` |
| One request | request body | `Batch` (building) / `Request` (built) |
| One reply | response body | `Response`, read through an `AnswerSheet` |

The word "rubric" is retired. The docs use it for one question's criteria; we don't need it.

---

## Design rules

Each rule closes one finding in `DIVERGENCES.md`.

1. **Every description slot is an `Entry`.** Instructions, option descriptions, level descriptions, Noul sides. Strings stay easy (`"..."` converts), objects are welcome (`json!({...})`), and the reply's `legend` is read as `Entry` too.
2. **A form contributes; a batch owns.** `Form::ask` writes into a sheet it was handed. Two forms, twenty forms, a loop of forms: one request.
3. **Forms can carry data.** A form is a value. A unit struct is a constant form; a struct with fields is a form built from a record. Both implement the same trait.
4. **A Score is a position.** The typed wrapper keeps the docs' `score` as its first field. Rounding is a method you call on purpose, next to `most_likely()` and the probabilities.
5. **Keys are prefixed, never retyped.** A form's field names are its local keys. The batch adds a prefix (`triage.area`, `dup.18.same_person`). The derive macro writes both the asking and the reading side, so a key is spelled once.
6. **Questions point into the state.** The form's instructions name the parts they judge with backticked paths, and the example state for a form lives next to it in a doc comment, so the two are written together.
7. **Keep the receipt.** Nothing in the crate discards `usage` or the model version. Every send can be recorded.
8. **One moment, one call.** The crate never sends twice for the same state on its own. A second request happens because a page turned or the state changed.

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
  let then = receipt.response.sheet().form("triage", &TriageForm)?;
  let now  = jevvy.fill(receipt.request.state.clone(), &NewRuleForm).await?;
  compare(then, now);
}
```

---

## Stages

Each stage lists its goal, the steps in the order to type them, a target snippet, and what "done" looks like. As in the first roadmap, every step is written by hand first; the macros in Stage 4 copy what the hand wrote.

### Stage 1: Wire ⏭️ NEXT

**Goal:** the crate speaks the API's exact dialect, with the docs' names, structure allowed everywhere the docs allow it, and a transport you can swap.

**Steps**

1. `types/entry.rs`. Keep `Entry` as `Text | Object | Array`, untagged. Keep the `From` impls. Add `From<bool>` and `From<f64>` that stringify, since the API takes text only, and a doc comment that says so.
2. `types/questions.rs`. Rename `NoulCriteria { yes, no }` to `{ when_true, when_false }` and keep the serde renames to `"true"` / `"false"`. Keep `ChoiceOption`. Add `Question::validate(&self) -> Result<(), JevvyError>`: a Choice has 1 to 255 options, a Score has 2 to 10 levels. New errors `TooManyOptions`, `BadLevelCount`.
3. `types/answers.rs`. Change `ScoreAnswer::legend` to `IndexMap<String, Entry>`. Wrap the answers map in a struct, `Answers(IndexMap<String, Answer>)`, with `get`, `iter`, `len`. Keep `Answer::kind()`.
4. `types/error.rs`. Fix the 422 message (it says "Unauthorized"). Add `Timeout`. Make `UnknownOption` and `UnknownLevel` carry the question key: `{ key, found }`.
5. `transport.rs`, new. `pub trait Transport { async fn send(&self, request: &Request) -> Result<Response, JevvyError>; }` and `pub struct Http { client: reqwest::Client, base_url, api_key }` that implements it with the existing retry loop. Honor `Retry-After` when present, else the doubling backoff. Add a default timeout of 30 s.
6. `client.rs`. `Jevvy<T: Transport = Http>`. Keep `new`, `from_env`, the setters, and add `timeout(Duration)` and `with_transport(T)`. `execute` becomes `send(&self, &Request)`, which validates every question first, then calls the transport.
7. Run `examples/manual.rs`. It should need only the rename from `execute` to `send`.

**Target**

```rust
let jevvy = Jevvy::from_env()?.timeout(Duration::from_secs(10));
let res = jevvy.send(&request).await?;
println!("{} tokens in, {} out, model {}", res.usage.input_tokens, res.usage.output_tokens, res.model);
```

**Done when** `manual` and `smoke` run against the real API, a Score with structured levels round-trips (send `json!({"what": ..., "examples": [...]})` as a level and read the legend back), and a Choice with 256 options is refused before it leaves the machine.

---

### Stage 2: Forms ⬜

**Goal:** typed questions and typed answers, with the docs' names, and a `Form` trait that writes into a sheet it is handed rather than owning a request.

**The shape:**

```
                 QuestionSheet (prefix "triage")            AnswerSheet (prefix "triage")
 TriageForm ──ask()──► noul("is_bug", ..)  ──► "triage.is_bug"   ◄── read() ◄── Triage { is_bug, area, severity }
                       choice::<Area>("area", ..)                      choice::<Area>("area")?
                       score::<Severity>("severity", ..)               score::<Severity>("severity")?
```

**Steps**

1. `traits.rs`, `Options`. Keep `ALL`, `name()`, `from_name()`. Change `describe()` to return `Option<Entry>`. Keep `question(instructions)`, now building an `Entry` per option.
2. `traits.rs`, `Levels`. Keep `ALL`, `from_index()`. Change `describe()` to return `Entry`. Keep `question()`. Move `nearest` off the trait and onto `Score<T>` (step 4).
3. `types/typed.rs`, `Choice<T>`. Fields `choice: T`, `probabilities: IndexMap<T, f64>`, `confidence: f64`. Keep the `TryFrom<&ChoiceAnswer>` bouncer; the error now names the key, so `TryFrom` takes a `(key, &ChoiceAnswer)` pair or the sheet passes the key in.
4. `types/typed.rs`, `Score<T>`. Fields `score: f64`, `probabilities: IndexMap<T, f64>`, `confidence: f64`. Methods: `nearest() -> T` (round, clamp), `most_likely() -> T` (argmax of probabilities), `normalized() -> f64` (score divided by the top level number), `at_least(level: T) -> bool` (score ≥ level's index). No `level` field.
5. `sheet.rs`, new. `QuestionSheet<'a> { prefix: String, questions: &'a mut Questions }` with `key(local) -> String` (joins with `.`; empty prefix means no join), and writers: `noul(key, instructions)`, `noul_with(key, instructions, NoulCriteria)`, `choice::<T: Options>(key, instructions)`, `score::<T: Levels>(key, instructions)`, `question(key, Question)`, and `form(key, &impl Form)`, which makes a child sheet with the joined prefix and calls the form's `ask`.
6. `sheet.rs`. `AnswerSheet<'a> { prefix: String, answers: &'a Answers }` with the mirror readers: `noul(key) -> Result<f64>`, `choice::<T>(key) -> Result<Choice<T>>`, `score::<T>(key) -> Result<Score<T>>`, `raw(key) -> Result<&Answer>`, and `form(key, &F) -> Result<F::Answers>`. `Response::sheet(&self) -> AnswerSheet<'_>` gives the root.
7. `traits.rs`, `Form`:
   ```rust
   pub trait Form {
     type Answers;
     fn ask(&self, sheet: &mut QuestionSheet);
     fn read(&self, sheet: &AnswerSheet) -> Result<Self::Answers, JevvyError>;
   }
   ```
   A constant form is a unit struct. A form built from data is a struct with fields. Same trait.
8. `examples/triage.rs`. Rewrite `Area` and `Severity` for the new `describe()`. Write `struct TriageForm;` and `struct Triage { is_bug, has_repro, area: Choice<Area>, severity: Score<Severity> }`, and `impl Form for TriageForm` by hand. Put the example state shape in the doc comment above the form (rule 6).
9. `client.rs`. `fill(&self, state, &form) -> Result<F::Answers>`: root sheet, ask, send, read. This replaces the old `ask::<T>`.

**Target**

```rust
/// Expects a state shaped like `{ "title": ..., "body": ... }`.
struct TriageForm;

impl Form for TriageForm {
  type Answers = Triage;
  fn ask(&self, s: &mut QuestionSheet) {
    s.noul("is_bug",    "Is `title` plus `body` a bug report, not a question or a feature request?");
    s.noul("has_repro", "Does `body` include steps to reproduce the problem?");
    s.choice::<Area>("area", "Which part of the project does `title` concern?");
    s.score::<Severity>("severity", "How badly does the problem in `body` affect users?");
  }
  fn read(&self, s: &AnswerSheet) -> Result<Triage, JevvyError> {
    Ok(Triage {
      is_bug:    s.noul("is_bug")?,
      has_repro: s.noul("has_repro")?,
      area:      s.choice("area")?,
      severity:  s.score("severity")?,
    })
  }
}

let t: Triage = jevvy.fill(issue, &TriageForm).await?;
println!("{:?} at {:.2}, severity {:.2} (nearest {:?})", t.area.choice, t.area.confidence, t.severity.score, t.severity.nearest());
```

**Done when** `triage` runs, and a second form built from data (`SamePerson { record }`, one Noul whose instructions are a `json!` object holding the record) fills correctly too.

---

### Stage 3: Batch ⬜

**Goal:** one request assembled from any number of forms, repeated forms, and loose questions; the reply read back through the same forms. This closes divergences 1 and 3 together.

**Steps**

1. `batch.rs`, new. `Batch<'a, T> { client: &'a Jevvy<T>, request: Request }`, made by `jevvy.ask(state)`. It is `#[must_use]`.
2. Chain methods that open a root `QuestionSheet` over `request.questions` and write: `.noul(key, ..)`, `.choice::<T>(key, ..)`, `.score::<T>(key, ..)`, `.question(key, q)`, `.form(prefix, &form)`, and `.each(prefix, forms)` where `forms` yields `(id, &form)` pairs and the key becomes `prefix.id`. `.model(name)` overrides the model for this request.
3. `.send(self) -> Result<Response>`.
4. `AnswerSheet::each(prefix, ids, &form)` returns `Vec<(id, F::Answers)>` for the repeated case. Sheet lookups use the same `prefix.id` join.
5. `examples/duplicates.rs`. A resume as state, a loop of `SamePerson` forms in one batch, then a threshold in code that lists likely duplicates. This is the docs' example, and it is the one the old typed route could not write.

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

**Done when** `duplicates` runs with N candidates and produces N answers under `dup.<id>.same_person` in one request, and the request JSON printed before sending shows every key exactly once.

---

### Stage 4: Derive ⬜

**Goal:** `#[derive(Form)]`, `#[derive(Options)]`, `#[derive(Levels)]`, generating exactly what Stages 2 and 3 wrote by hand. Structured descriptions are first-class in the attributes.

**Steps**

1. Turn the project into a workspace. New crate `jevvy-derive` (proc-macro) with `syn`, `quote`, `proc-macro2`. Re-export the macros from `jevvy` behind a default `derive` feature.
2. `#[derive(Options)]` on an enum. Reads `#[name("...")]` (optional, default snake_case of the variant) and `#[describe(...)]` in two forms: a string literal, or key = value pairs (`what = "...", not_for = "...", examples = ["..", ".."]`), which become an `Entry::Object` with those keys. Writes `ALL`, `name()`, `describe()`.
3. `#[derive(Levels)]` on an enum. Same `describe` forms. Writes `ALL` in declaration order and `describe()`.
4. `#[derive(Form)]` on the *answers* struct. Reads `#[noul(...)]`, `#[choice(...)]`, `#[score(...)]` per field; each takes a string literal or key = value pairs for structured instructions, and `noul` also accepts `when_true = ...`, `when_false = ...`. Generates a unit struct named `<Name>Form` and `impl Form for <Name>Form { type Answers = <Name>; ... }` whose keys are the field names. The field types decide the reader: `f64`, `Choice<T>`, `Score<T>`.
5. `trybuild` tests: a `Choice<T>` field where `T` is not `Options`, a struct with an unannotated field, a `describe` with an unknown key. Each must fail with a message that names the field.
6. Switch `examples/triage.rs` and `examples/duplicates.rs` to the derives, and compare the generated request JSON to Stage 3's byte for byte.

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

**Done when** the derived examples produce the same JSON as the hand-written ones and the `trybuild` errors read like advice.

---

### Stage 5: Gates ⬜

**Goal:** the three paths from the docs (act, confirm, escalate) as a small typed vocabulary, with every threshold still in your code. This is the "unsure door".

**Steps**

1. `gate.rs`, new. `pub enum Verdict<T> { Act(T), Confirm(T), Escalate }` with `is_sure()`, `map()`, and `into_option()`.
2. `Gate { act: f64, confirm: f64 }`. `judge(&self, &Choice<T>) -> Verdict<&T>`: confidence at or above `act` acts, at or above `confirm` confirms, otherwise escalates. `judge_score(&self, &Score<T>) -> Verdict<f64>` gates on confidence and hands back the position.
3. `NoulGate { yes: f64, no: f64 }`. `judge(p: f64) -> Verdict<bool>`: above `yes` is `Act(true)`, below `no` is `Act(false)`, the band between is `Escalate`. The docs' 0.8 / 0.2 split.
4. Gates are `const`-constructible so they sit next to the code that uses them, named for the action: `const APPROVE_TRANSFER: Gate = Gate { act: 0.85, confirm: 0.6 };`.
5. `examples/gates.rs`. The voice-banking example: one Choice, two gates by stakes, and the printed verdict for a handful of commands.

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

**Done when** `gates` prints a different verdict for the same confidence under the two gates, and a Noul in the unsure band comes out as `Escalate`.

---

### Stage 6: Ledger and replay ⬜

**Goal:** every send can be kept as a receipt; a ledger of receipts can stand in for the API (tests with no network) or be walked again with new questions (backtests). This is also where the first real unit tests arrive.

**Steps**

1. `receipt.rs`, new. `Receipt { at_ms: u64, request: Request, response: Response }`, `Serialize + Deserialize`. `at_ms` is Unix milliseconds from `SystemTime`, so no new dependency.
2. `ledger.rs`, new. `Ledger` over a JSON Lines file: `open(path)` (creates if missing), `append(&Receipt)`, `iter() -> impl Iterator<Item = Result<Receipt>>`, `len()`.
3. `transport::Recording<T>`. Wraps any transport; on every successful send it appends a receipt to a ledger. `jevvy.recording("receipts.jsonl")?` returns a `Jevvy<Recording<Http>>`.
4. `transport::Replay`. Loads a ledger into a map keyed by the request serialized to a canonical JSON string. `send` returns the recorded response or `JevvyError::NotRecorded`. `Jevvy::replay(path)`.
5. `tests/triage.rs`. Record `examples/triage.rs` once with `Recording` into `fixtures/triage.jsonl`, then a test that runs the same form through `Replay` and asserts on the typed answers. Same for `duplicates`. This is the mock transport the first roadmap parked.
6. `examples/replay.rs`. Open a ledger, and for each receipt send its *state* with a *new* form (a changed threshold, a reworded level), then print old verdict next to new verdict. A backtest in twenty lines.

**Target**

```rust
let jevvy = Jevvy::from_env()?.recording("receipts.jsonl")?;   // every send is kept

// later, offline
let jevvy = Jevvy::replay("fixtures/triage.jsonl")?;
let t: Triage = jevvy.fill(issue, &TriageForm).await?;          // served from the file

// later still, a backtest
for r in Ledger::open("receipts.jsonl")?.iter() {
  let r = r?;
  let then = r.response.sheet().form("triage", &TriageForm)?;
  let now  = live.fill(r.request.state.clone(), &TriageV2Form).await?;
  println!("{:?} -> {:?}", ROUTE.judge(&then.team), ROUTE.judge(&now.team));
}
```

**Done when** `cargo test` passes with no network and no API key, and `replay` prints a before/after column for every receipt in a ledger.

---

### Stage 7: Watch ⬜

**Goal:** a state your program mutates, a panel of standing questions, and a `tick()` that sends one batch only when the state changed, returning the answers and a diff against the last reading. Standing questions become sensors. This is "it notices" and "it moves only on change".

**Steps**

1. `diff.rs`, new. `Change { key: String, kind: ChangeKind }` with `ChangeKind::{ Noul { from, to }, Choice { from, to }, Score { from, to }, Confidence { from, to }, Added, Removed }` and `magnitude() -> f64` (absolute delta for numbers, 1.0 for a changed choice, 0.0 otherwise). `diff(before: &Answers, after: &Answers) -> Vec<Change>`, keyed match.
2. `watch.rs`, new. `Watch<'a, T, S, F>` where `S: Serialize` and `F: Form`, made by `jevvy.watch(state, form)`. Holds `state`, `form`, `dirty: bool`, `last: Option<Receipt>`.
3. `update(&mut self, f: impl FnOnce(&mut S))` runs `f` and marks dirty. `set(S)` replaces. `state(&self) -> &S`.
4. `tick(&mut self) -> Result<Option<Reading<F::Answers>>>`. Not dirty: `Ok(None)`, no request. Dirty: serialize state to `Entry`, batch the form at the root, send, diff against `last`, store the receipt, clear dirty, return `Some(Reading { answers, changes, receipt })`. `force()` ticks even when clean.
5. `Reading::alarms(min_magnitude) -> impl Iterator<Item = &Change>`.
6. `examples/inbox.rs`. A conversation struct as state. Push six messages one at a time from a script, tick after each, print the frustration score, the team, and every change above 0.3. Then push the same message twice and show the second tick returns `None`.

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

**Done when** `inbox` shows one request per changed state, zero for an unchanged one, and a printed diff line when frustration climbs.

---

### Stage 8: Walk ⬜

**Goal:** the book. Code writes pages; each page asks a batch, then turns to another page, finishes, or admits it is unsure. Jev picks at every fork; code turns the page; the trail of receipts is the record. This generalizes the docs' taxonomy walk and skill-suggestion cookbook.

**Steps**

1. `walk.rs`, new.
   ```rust
   pub trait Page {
     type Outcome;
     fn ask(&self, sheet: &mut QuestionSheet);
     fn turn(&self, sheet: &AnswerSheet) -> Result<Turn<Self::Outcome>, JevvyError>;
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
4. `examples/taxonomy.rs`. `DepartmentPage` asks one Choice over top-level departments, each option's description being the subtree (rule 1 makes that a `json!` object). `turn` gates on confidence: `Act` goes to `SubIssuePage { department }`, `Confirm` also goes on but marks the trail, `Escalate` returns `Unsure`. The leaf page returns `Done(path)`.
5. Print the trail: pages visited, the chosen option and confidence at each, and total tokens from the receipts.

**Target**

```rust
struct DepartmentPage;
impl Page for DepartmentPage {
  type Outcome = Vec<Team>;
  fn ask(&self, s: &mut QuestionSheet) { s.choice::<Department>("department", "Which department does `listing` belong to?"); }
  fn turn(&self, s: &AnswerSheet) -> Result<Turn<Vec<Team>>, JevvyError> {
    let d = s.choice::<Department>("department")?;
    Ok(match CLASSIFY.judge(&d) {
      Verdict::Act(dep) | Verdict::Confirm(dep) => Turn::Next(Box::new(SubIssuePage { department: *dep })),
      Verdict::Escalate => Turn::Unsure(format!("department split: {:?}", d.probabilities)),
    })
  }
}

let trail = jevvy.walk(json!({ "listing": listing }), Box::new(DepartmentPage)).await?;
```

**Done when** `taxonomy` reaches a leaf in two requests for a clear listing, stops with `Unsure` for an ambiguous one, and the trail lists both.

---

### Stage 9: Polish and publish ⬜

- ⚠️ `.env` was in the first commit. Rotate the key or rewrite history before adding a remote.
- README: the vision in three paragraphs, then the day-in-the-life snippet, then one example per stage.
- Doc comments with runnable examples on every public item; `cargo doc` clean; `cargo test` runs the doctests through `Replay` fixtures.
- Fix the leftover typos listed in the first roadmap.
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
- Give an overview first, then go step by step.
- Keep explanations in simple English, with analogies and diagrams.
- Save issues outside the current file for a checkpoint instead of raising them mid-step.
- Style: 2-space indent, doc comments on public items.
- Claude doesn't run git commands on my machine.
- Every stage ends with a checkpoint: run its example, then update the status table and `_Last updated_` here.

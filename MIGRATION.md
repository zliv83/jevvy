# Migration: from the Stage 4 crate to ROADMAP2

> A reference, not a task list. `ROADMAP2.md` is what you follow; this file says, file by file, what of today's code survives it. Read it once to get the picture, then again at each stage.

## The picture

```
 TODAY                                              ROADMAP2

 state ─► RequestBuilder ─┐                         state ─► Batch ◄── Form, Form, each(Form…), loose q
          (string keys)   ├─► Request ─► execute    (prefixed keys)      │
 state ─► Rubric::questions()               │                            ▼
          (one type = one request)          │                   Request ─► Jevvy::send ─► Transport (Http | Recording | Replay)
                                            ▼                                                      │
                                       Response                                               Response ─► Receipt ─► Ledger
                                        │     │                                                  │
                     res.noul/choice/score   T::from_response                              AnswerSheet ─► Form::read ─► typed answers
                              │                  │                                                             │
                              └──── your code ◄──┘                                        Gate ─► Verdict ─► your code
                                                                                          Watch (tick, diff)   Walk (pages, trail)
```

The middle is the same pipe. What changes: forms contribute to a batch instead of owning a request; the transport is swappable and recordable; answers are read through a prefixed sheet; and three new layers (gates, watch, walk) sit on top for a running program.

## Name map

| Today | ROADMAP2 | Stage |
|---|---|---|
| `Jevvy::execute` | `Jevvy::send` | 1 |
| `Jevvy::evaluate` | `Jevvy::ask` (returns a `Batch`) | 3 |
| `Jevvy::ask::<T>` | `Jevvy::fill(state, &form)` | 2 |
| `RequestBuilder` | `Batch` | 3 |
| `Rubric` | `Form` (has `type Answers`, `ask`, `read`) | 2 |
| `Rubric::questions() -> Questions` | `Form::ask(&self, &mut QuestionSheet)` | 2 |
| `Rubric::from_response(&Response)` | `Form::read(&self, &AnswerSheet)` | 2 |
| `Response::noul/choice/score(key)` | `AnswerSheet::noul/choice/score(key)` (root sheet via `Response::sheet()`) | 2 |
| `Choice<T>::value` | `Choice<T>::choice` | 2 |
| `Score<T>::raw` | `Score<T>::score` | 2 |
| `Score<T>::level` | `Score<T>::nearest()` (a method) | 2 |
| `Levels::nearest(raw)` | `Score<T>::nearest()` | 2 |
| `Options::describe() -> Option<&'static str>` | `-> Option<Entry>` | 2 |
| `Levels::describe() -> &'static str` | `-> Entry` | 2 |
| `NoulCriteria { yes, no }` | `{ when_true, when_false }` | 1 |
| `ScoreAnswer::legend: IndexMap<String, String>` | `IndexMap<String, Entry>` | 1 |
| `Answers` (type alias) | `Answers` (struct wrapper) | 1 |
| `UnknownOption(String)` | `UnknownOption { key, found }` | 1 |

## File by file

### Keep as-is

| File | Why |
|---|---|
| `src/lib.rs`, `src/types.rs` | Module lists; they only grow. |
| `src/types/entry.rs` | `Entry` is exactly the docs' `EntryType`. Rule 1 makes it the type of every description slot, so it becomes *more* central. Stage 1 adds two `From` impls. |
| `src/types/request.rs` | `Request { state, model, questions }` is the wire shape. Untouched. |
| `examples/manual.rs` | The wire-level demo. One rename, `execute` → `send`. |
| `ROADMAP.md` | History of Stages 1–4 and the lessons in it. Leave it. |
| `docs/` | The TypeSafe vault. Untouched. |

### Alter a little

| File | Change | Stage |
|---|---|---|
| `src/types/questions.rs` | Rename the two `NoulCriteria` fields (serde renames stay). Add `Question::validate`. `ChoiceOption`, `Question::noul/choice/score`, `Questions` all stay. | 1 |
| `src/types/answers.rs` | `legend` becomes `IndexMap<String, Entry>`. `Answers` becomes a small struct around the map so a sheet can borrow it. `Answer::kind()` stays. | 1 |
| `src/types/response.rs` | Keep the struct and `Usage`. The three getters move onto `AnswerSheet`; `Response::sheet()` returns the root sheet. `wrong_type` moves with them. | 2 |
| `src/types/error.rs` | Fix the 422 message. Add `Timeout`, `TooManyOptions`, `BadLevelCount`, `NotRecorded`. Give `UnknownOption`/`UnknownLevel` a `key`. `handle_response` stays and moves to `transport.rs`. | 1, 6 |
| `src/types/typed.rs` | Rename fields to the docs' words; drop `level`; add `nearest()`, `most_likely()`, `normalized()`, `at_least()`. The two `TryFrom` bouncers and `lookup`/`level_at` stay, now told the key so errors can name it. `confident(min)` can stay as a convenience; gates replace it in practice. | 2 |
| `src/traits.rs` | `Options` and `Levels` stay with `describe()` widened to `Entry`. `nearest` moves off `Levels`. `Rubric` is replaced by `Form` (see Discard). | 2 |
| `src/client.rs` | Keep `new`, `from_env`, the setters, `DEFAULT_*`, `backoff`. The retry loop moves into `transport::Http`. `Jevvy` grows a type parameter with a default, `timeout`, `with_transport`, `recording`, `replay`, `fill`, `ask`, `watch`, `walk`. | 1, 2, 3, 6, 7, 8 |
| `src/builder.rs` | Becomes `src/batch.rs`. The chain methods, `#[must_use]`, `model`, and `send` stay word for word; `question` now goes through a root `QuestionSheet`; `form` and `each` are added. | 3 |
| `examples/smoke.rs` | `evaluate` → `ask`; getters through `res.sheet()`. Otherwise the same. | 3 |
| `examples/triage.rs` | `Area` and `Severity` keep their variants and text, wrapped in `Entry`. `Triage` keeps its fields; the `Rubric` impl becomes `impl Form for TriageForm` with `ask`/`read` on a sheet. `labels_for` changes `.value` → `.choice` and `.level ==` → `.nearest() ==`. Later, Stage 4 replaces the hand impl with `#[derive(Form)]`. | 2, 4 |
| `Cargo.toml` | Becomes a workspace root at Stage 4; `jevvy-derive` gets `syn`, `quote`, `proc-macro2`; `trybuild` in dev-deps. No new runtime dependencies through Stage 8. | 4 |

### Discard

| What | Why | Replaced by |
|---|---|---|
| `trait Rubric` | One type owned one request; `questions()` had no `self`, so no data could reach it; keys were typed twice. | `Form` with `ask(&self, sheet)` / `read(&self, sheet)` |
| `Jevvy::ask::<T: Rubric>` | Same reasons. | `fill(state, &form)` for one form, `ask(state).form(..).send()` for many |
| `Score<T>::level` field | Headlined the rounded reading over the position. | `score` field plus `nearest()` |
| `Levels::nearest` on the trait | Belongs to the answer, not the enum. | `Score<T>::nearest()` |
| `Response::noul/choice/score` as methods on `Response` | Needed a prefix to serve forms. | `AnswerSheet` |
| The `Stage 5` plan in `ROADMAP.md` (`#[derive(Rubric)]`, keyed by field name, string-literal attributes only) | Would have frozen divergences 1, 2 and 3 into generated code. | ROADMAP2 Stage 4, after `Form` and `Batch` exist |

## What each finding in DIVERGENCES.md turns into

| Finding | Closed by |
|---|---|
| 1. printed form vs data-driven | `Form` takes `&self`, so a form can carry a record; `Batch::each` stamps them out. |
| 2. one-line labels | `Entry` in every description slot; derive attributes take key = value pairs. |
| 3. one rubric per call | `Batch` with prefixed keys; forms contribute. |
| 4. nearest level as headline | `Score<T>::score` first; `nearest()` is a method beside `most_likely()`. |
| 5. freestanding form | Instructions name paths; the expected state shape lives in the form's doc comment; `Watch` ties one state type to one form. |
| 6. vocabulary | Docs' words on every wire and typed field; "rubric" retired. |
| receipts thrown away | `Receipt`, `Ledger`, `Recording`. |
| no timeout | `Http` default 30 s, `timeout()` setter. |

## Order of operations

1. Stage 1 first, and it is almost all edits to existing files. After it, `manual` and `smoke` run again.
2. Stage 2 is the only stage that deletes anything (`Rubric`). Do it in one sitting so `triage` is never broken across two sessions.
3. Stages 3 through 8 only add files.
4. `CLAUDE.md` (untracked) should point at `ROADMAP2.md` instead of `ROADMAP.md` once Stage 1 begins.

## Tally

Roughly: eleven of fourteen source and example files survive with edits, three things are removed (`Rubric`, `ask::<T>`, the `level` field), and the retry loop, `Entry`, the question constructors, the answer types, both enums, and the builder's chain methods are kept word for word.

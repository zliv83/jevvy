# Where jevvy (Stages 1–4) and the TypeSafe docs part ways

> Reference for `ROADMAP2.md` and `MIGRATION.md`. The interactive version, with a playground per item, is at https://claude.ai/artifact/8K68yBJ4jptb4YWp919ZwE

**What already lines up.** Three primitives with the API's exact shapes, ordered maps included. One state, many questions, one call. Probabilities kept on every answer. Thresholds in your code. Question ids never reach the model. Null descriptions when the name is enough. Retries on 429/529 with backoff.

The drift is almost entirely in the typed route, and it is about what gets frozen at compile time.

## 1. Questions from data, or a printed form

The docs stamp questions out of data at run time (one Noul per candidate record, options from a taxonomy branch, one battery per extracted field). A `Rubric` struct is a printed form: fields, keys and options are fixed when the program compiles. The builder route can do the data-driven case, but the two routes don't meet.

*Why:* Rust fixes types before the program runs, and Stage 4 traded string keys for fields. That fits the fixed fan-out battery and not the data-driven battery.

## 2. Structured rubrics, or one-line labels

The docs' mature question is an object with labelled parts: a `question`, a `focus`, the data it points at; options with `what`, `not_for`, `examples`; levels with `what` and `signals`; a Noul with a `true` side and a `false` side. In jevvy, `describe()` returns one `&'static str`, a Noul inside a rubric has nowhere to put criteria, and the planned attributes take a string literal. The reply reader also assumes a level's legend is text, so a structured Score level can't be read today.

*Why:* the traits were modelled on the first example on each docs page, and an enum variant maps naturally to one sentence.

## 3. Everything in one call, or one rubric per call

The docs put every question about a state into one request ("add another question; the request count stays at one"). A `Rubric` is one request's worth, and rubrics don't combine. Reuse means one giant struct or extra round trips.

*Why:* `Rubric` was designed as "the whole form", so one type is one request.

## 4. A position on a line, or the nearest level

The docs read `score` as a weighted position you threshold, rank, or normalize and weight; rounding to a level is the last option they list. `Score<T>` names the rounded variant `level` first and the position `raw` second, and `nearest` always picks with no look at the spread. A 50/50 Trivial/Blocker split rounds to Annoying, a level the model never chose.

*Why:* an enum wants a variant, and "your own type back" was the promise.

## 5. Questions that point into the state, or a freestanding form

The docs write state and questions as a pair: an object with named parts, and questions that name the part they judge with a backticked path. `ask::<T>(anything)` treats the form as freestanding; hand it a different shape and the paths point at nothing, and the compiler is happy either way.

*Why:* "separate content from questions" was taken literally, and the second half, "point your questions at named parts", has no home.

## 6. Vocabulary drift

| TypeSafe says | jevvy says |
|---|---|
| `choice` (the picked option) | `value` |
| `score` (the position) | `raw` |
| no field | `level` |
| `true` / `false` (Noul criteria) | `yes` / `no` |
| "rubric" (one question's criteria) | `Rubric` (the whole question set) |

*Why:* `true` is a keyword, `choice.choice` reads badly, and "Rubric" was a friendly umbrella.

## Smaller

- `ask` throws away the receipt (usage tokens and model version).
- The builder's Noul can't take criteria.
- No HTTP timeout.

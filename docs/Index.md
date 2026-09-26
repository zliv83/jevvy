---
aliases:
  - Start here
---

> Read this note first. It maps every note in this vault so you don't have to discover the structure yourself.

## What this vault is

This is a partial mirror of the **TypeSafe / Jev** product docs (`docs.typesafe.ai`), imported into Obsidian. TypeSafe's "System One" models (Jev) take a `state` and a set of typed `questions` ([[Primitives (Questions)|Choice, Score, Noul]]) and return typed, probability-backed `answers` that software can act on directly.

Only the pages below exist as notes in this vault. Links to pages the site has but this vault doesn't (cookbooks, demos, SDK reference, `/models`, `/agent-skill`, `/introduction/*`) were rewritten as plain external links to `https://docs.typesafe.ai/...` rather than left as dead internal links — click through if you need that material. [[llms]] is the full site's link index and is all external links by design; leave it as-is.

## Suggested reading order

1. [[System One]] — what a System One model is and why it's not an agent.
2. [[State]] — the input you send a model.
3. [[Primitives (Questions)]] — the three question types and how to choose between them, then the type-specific notes: [[Choice]], [[Score]], [[Noul]], [[Advanced (structure)]].
4. [[Confidence]] — reading and gating on model certainty.
5. [[How to build with TypeSafe]] — end-to-end architecture guidance.
6. [[Patterns]] — composable patterns built from the primitives: [[Speculative fan-out]], [[Confidence-gated routing]], [[Composite scoring]], [[Intent routing]].
7. [[API reference]] — the HTTP request/response schema.
8. [[AI Primer]] and [[Model Jaggedness]] — background and known model limitations, read as needed.

## Concepts

| Note | What it covers |
|---|---|
| [[System One]] | System One models make fast, structured decisions for software; Jev is TypeSafe's flagship model. |
| [[State]] | What state is, how to structure it, and how to give a model the context it needs. |
| [[Primitives (Questions)]] | The three question types (Choice, Score, Noul), their typed answers, and how to ask several at once. |
| [[Choice]] | Selects one option from a defined set; returns the option, a probability per option, and confidence. |
| [[Score]] | Rates content against ordered, descriptive levels; returns a score, a probability per level, and confidence. |
| [[Noul]] | Evaluates a yes/no question; returns the probability that the answer is yes. |
| [[Advanced (structure)]] | How Choice options, Score levels, Noul criteria, and instructions accept JSON structure instead of plain strings. |
| [[Confidence]] | How TypeSafe reports certainty, how it differs from probability, and how to use it to control system behavior. |
| [[How to build with TypeSafe]] | Designing AI-powered software by keeping code in control and giving System One narrow, structured decisions. |
| [[AI Primer]] | Why TypeSafe trains decision models with calibrated probabilities instead of optimizing for generated text. |

## Patterns

| Note | What it covers |
|---|---|
| [[Patterns]] | Index of architectural patterns for building systems with TypeSafe. |
| [[Speculative fan-out]] | Send many questions in one call, including speculative ones, and let code decide what's relevant. |
| [[Confidence-gated routing]] | Use confidence as a second decision axis to build safer systems. |
| [[Composite scoring]] | Break a complex judgment into independent scores, combine with weights controlled in code. |
| [[Intent routing]] | Classify a user's intent and route to the appropriate handler (logic, LLM, or human). |

## Reference

| Note | What it covers |
|---|---|
| [[API reference]] | Full HTTP API reference for the TypeSafe evaluation endpoint. |
| [[Model Jaggedness]] | Known jagged edges / failure modes of the current Jev model (jev-1.13). |
| [[llms]] | Full site map for the live docs, as external links — not vault notes. |

## Link conventions in this vault

- Internal links use `[[Wikilinks]]`, resolved by note title (all titles in this vault are unique, so no folder prefix is needed).
- A few links point at a specific spot inside a note using a block reference (`[[Note#^block-id]]`) rather than a heading, because the source page used a UI component (a `<Step>` or `<ParamField>`) with no real Markdown heading to link to.
- Links to material outside this vault's 17 notes go straight to `https://docs.typesafe.ai/...`.

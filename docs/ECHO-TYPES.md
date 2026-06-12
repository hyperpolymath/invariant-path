<!-- SPDX-License-Identifier: MPL-2.0 -->
<!-- SPDX-FileCopyrightText: 2026 Jonathan D.A. Jewell -->

# Invariant Path × echo-types — structured loss as a first-class object

Invariant Path is the **application example** of the
[`echo-types`](https://github.com/hyperpolymath/echo-types) programme: it is a
runtime that *keeps echoes instead of discarding them*.

## The correspondence

echo-types formalises **fiber-based structured loss**. For a function
`f : A → B`, the *echo* of an output `y` is the fiber

```
Echo f y  :=  Σ (x : A) , (f x ≡ y)
```

— the proof-relevant record of **which** inputs `f` collapsed onto `y`. A
non-injective `f` *forgets* that "which"; the echo is exactly what you must
retain to recover it.

Invariant Path's classifier is such an `f`:

```rust
classify_candidate : ClaimCandidate -> ClassificationOutcome
```

Many distinct claim-paths collapse to the same `Classification`
(`ValidPath | Overextended | Conflation | Incomplete | Abstain`). The bare
classification is the lossy codomain value; **the claim-path is the echo**.

| echo-types                | Invariant Path                                   |
|---------------------------|--------------------------------------------------|
| `f : A → B` (lossy map)   | `classify_candidate` (claim-path → classification) |
| codomain value `y`        | `Classification`                                 |
| `Echo f y` (the fiber)    | the retained `ClaimCandidate` + `ClassificationOutcome` |
| residue / structured loss | `ClassificationOutcome.losses : Vec<String>`     |
| no-section (irreversible) | a `Classification` alone cannot recover its path |
| the section that recovers | `Annotation.path_description` (retained provenance) |

The domain model already names the loss: `ClassificationOutcome` carries
`losses` and `preserved` precisely because a classification that *dropped* the
path would be untraceable. echo-types is the mechanised backbone of that design
choice — Invariant Path is "a claim-path debugger, not a truth engine" exactly
because it stores echoes.

## Worked example (runnable + tested)

```bash
cargo run -p invariant-path-core --example echo_structured_loss
cargo test -p invariant-path-core --test echo_structured_loss
```

Two genuinely different claim-paths —
`benchmark accuracy ⟶ general capability` and
`the theorem proves ⟶ production guarantee` — both classify as `Overextended`,
yet retain **distinct** structured losses (`task_transfer` vs
`implementation_gap`). The classifier is non-injective; the echo keeps what it
drops. See:

- `crates/invariant-path-core/examples/echo_structured_loss.rs`
- `crates/invariant-path-core/tests/echo_structured_loss.rs`

## Cross-repo

The same `Echo` underlies the type-system integration across the estate, all
machine-checked under `--safe --without-K`:

- **nextgen-typing** — `verification/proofs/agda/EchoTyping.agda`: affine
  subtyping *is* echo `weaken`; refinement erasure *is* a fiber.
- **nextgen-languages / kitchenspeak** — `proofs/agda/EchoBridge.agda`: the `@`
  sensor witness *is* `Echo (fired …) true`.
- **phronesis** — `academic/formal-verification/agda/PhronesisEcho.agda`: an
  ethical verdict's provenance *is* `Echo verdict v`.

Invariant Path is where that theory is *used*: a tool whose reason to exist is
to retain the echo.

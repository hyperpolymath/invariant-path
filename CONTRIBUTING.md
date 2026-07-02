<!--
SPDX-License-Identifier: CC-BY-SA-4.0
SPDX-FileCopyrightText: 2026 Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk>
-->

# Contributing to Invariant Path

Thanks for your interest. This is a small, focused MVP maintained by one
person; contributions that keep it small are the most welcome kind.

## Getting started

```bash
cargo build --workspace
cargo test --workspace
```

No external services or toolchains are needed for the core; the AffineScript
`faces` examples and the Agda proof are optional extras that degrade to SKIP
when their toolchains are absent.

## What's useful

- Bug reports with a minimal input text and the annotation you expected
- New extraction triggers or classifier heuristics, **with tests** (see
  `crates/invariant-path-core/src/lib.rs` for the test style) and an entry in
  `docs/EXTENDING.md`
- Precision/recall evaluation corpora — the biggest known gap

## Ground rules

- Code is MPL-2.0, docs are CC-BY-SA-4.0; keep SPDX headers on new files
- `#![forbid(unsafe_code)]` stays
- `cargo test --workspace` must pass; `cargo fmt` before committing
- Heuristics favour false negatives over noise (see `DESIGN.md`) — a change
  that makes the tool chattier needs a strong justification

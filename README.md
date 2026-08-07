<!--
SPDX-License-Identifier: CC-BY-SA-4.0
SPDX-FileCopyrightText: 2025-2026 Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk>
-->

[![OpenSSF Best Practices](https://img.shields.io/badge/OpenSSF-Best_Practices-green?logo=opensourcesecurity)](https://www.bestpractices.dev/en/projects/new?repo_url=https://github.com/hyperpolymath/invariant-path)
[![License: MPL-2.0](https://img.shields.io/badge/License-MPL--2.0-blue.svg)](LICENSE) <embed
src="https://api.thegreenwebfoundation.org/greencheckimage/github.com"
data-link="https://www.thegreenwebfoundation.org/green-web-check/?url=github.com" />

# Invariant Path

**Invariant Path traces what a claim *means* when it crosses from one domain
into another — and finds the crossings where the meaning quietly changed.**

It is a claim-path debugger, not a truth engine. It does not decide whether a
claim is true. It shows you the path a claim took, and where along that path it
stopped meaning what it meant at the start.

## The idea

A claim is asserted in some domain. It is then re-expressed in another — a
specification lowered into code, a licence text asserting facts about its own
file tree, a source language emitted through several backends, a proof
obligation discharged in a different logic.

Two things can be true at once:

* the claim holds on **both** sides of the transition, and
* it does **not mean the same thing** on both sides.

That second case is the one this tool exists for. Nothing on the surface
signals it: both sides pass their own checks, both look green, and the
divergence is invisible precisely because each side is internally consistent.
The invariant is not "the claim is true here and true there" — it is
**"the claim is the *same claim* here and there."** That is the path, and the
path is what has to be invariant.

### The failure is silent, which is why it needs tooling

Per-domain tests are the wrong instrument by construction. A test inside domain
A checks A against itself; a test inside domain B checks B against itself.
Neither can see that A's claim and B's claim have drifted apart, because
**cross-domain equality is not expressible from inside either domain.** Every
individual test can pass forever while the thing you actually care about is
already broken.

### The worked example: "different faces, same cube"

AffineScript presents several *faces* — surface syntaxes — that all lower to
one canonical form. Per-face snapshot tests catch drift *within* a face and
never compare face A's cube against face B's. **The cross-face equality is the
load-bearing claim**, and no per-face test can state it.

Invariant Path grounds that claim: each face is compiled to typed-wasm and the
modules are compared by `sha256`, so the faces must land in **one wasm
equivalence class**. The wasm *is* the cube, which makes this a far stronger
bar than matching canonical text. When a face falls outside the class, a
normalised text diff is printed to locate *where* — as a diagnostic, not as the
check itself.

This is one worked example, not the subject of the repository. See
`profiles/faces.md` for its limits, including the two classes the `greet`
corpus genuinely splits into.

## The three-layer claim checker

Invariant Path is the **governance front-end** for a layered checker. Each
layer answers a question the others cannot:

| Layer | Name | Question |
|---|---|---|
| 1 | **type** | Is this admissible *on form alone*? Well-formedness, through-line, and a forward-only ordering on evidence — "the diode". |
| 2 | **trope** | Does this particular evidence survive the leap to this conclusion *for this use*? Purpose-indexed warrant. |
| 3 | **sortal** | Are these two presentations *the same argument*? Identity, and therefore counting. |

Layer 3 was ruled on in
[ADR-0001](docs/decisions/0001-the-sortal-layer.adoc) after a five-count case
against it was built deliberately and three of the five counts broke. Its
killer case is the **doubling attack**: one derivation presented twice,
paraphrased, under distinct labels is invisible to every Layer 1 and Layer 2
check — identical pass/fail on every subdiagram — while corroboration weight is
two instead of one. That is why Layer 3 runs *before* Layer 2: identity must be
settled before resource accounting, or Layer 2 double-counts.

**These layers are decided, not built.** The ADR is a decision record. What
ships today is the extractor, classifier and annotation store described below.
Open architectural gaps are tracked in [`docs/DEBT.md`](docs/DEBT.md).

## Its dual: 007 and hermeneutic semantics

Invariant Path and the hermeneutic-semantics work in `007` are two halves of one
concern, and each is the other's mirror:

| | Syntax | Semantics | The silent failure |
|---|---|---|---|
| **Invariant Path** | **different** | must be **the same** | forms diverge in meaning while each stays self-consistent |
| **007 / hermeneutic** | **the same** | may be **read differently** | one form silently carries two readings, and the wrong one is assumed |

*Many forms, one meaning* against *one form, many meanings*. Both fail the same
way — silently, because the surface text carries no signal of the divergence.
If you are working on one, read the other; a fix in either that ignores the
dual will be incomplete.

## What a profile is

A **profile** is a lens this tool applies to a corpus that lives **somewhere
else**. It names a target corpus by path, states the claim being traced, and
supplies whatever verifier grounds it.

**A profile does not vendor the corpus it examines.** See `profiles/pmpl.md`
for the intended shape: it points at the palimpsest-license tree and copies
nothing. A profile that carries a copy of its target will drift from the real
thing and quietly start grounding a claim about a stale fixture — which is this
tool's own failure mode, turned inward.

# Workspace Layout

- `crates/invariant-path-core` — extractor, classifier, schema models,
  and annotation storage API

- `crates/invariant-path-cli` — CLI/TUI-adjacent interface for scan +
  annotation editing

- `schemas/annotation.schema.json` — JSON schema for persisted
  annotations

- `docs/ARCHITECTURE.md` — minimal architecture proposal

- `docs/EXTENDING.md` — extension guide for invariant types and
  heuristics

- `examples/` — seeded examples. NOTE: `examples/same-cube/` currently
  vendors an AffineScript corpus, which contradicts the profile rule
  above; see the open issue to point it at the real tree instead.

- `profiles/` — profile notes for `echidna`, `panll`, `hypatia`, `pmpl`,
  `standards-docs`, and `faces` (AffineScript "different faces, same
  cube" — pending rename to `affinescript`, matching the others, which
  are named for their target corpus rather than for one claim inside it)

- `scripts/verify-same-cube.sh` — grounds the faces same-cube invariant
  (see `profiles/faces.md`)

# Quick Start

```bash
cargo run -p invariant-path-cli -- scan --file ./README.md --artifact-uri repo://README.md --json
```

```bash
cargo run -p invariant-path-cli -- annotations list --json
```

```bash
cargo run -p invariant-path-cli -- doc-claims scan --file ./README.md --json
```

# CLI Surface

The CLI currently provides five subcommands:

- `scan` — extract and classify claim transitions from a file
- `annotations` — list/add/update/accept/dismiss/clarify persisted annotations
- `overlay` — toggle overlay state
- `profiles` — list built-in scan profiles (`generic`, `echidna`, `panll`, `hypatia`)
- `doc-claims` — ground factual doc claims (file paths, command hygiene) against the filesystem

There is no interactive TUI yet; all interaction is CLI-first with `--json` output.

# Launcher Integration (maintainer machine only)

The `./invariant-path` and `./invariant-path-launcher` wrappers, the desktop
template (`desktop/invariant-path.desktop`), and `scripts/install-desktop.sh`
delegate to a launcher script that lives outside this repository
(`/var/mnt/eclipse/repos/.desktop-tools/`). On any other machine they print a
clear message and exit; use `cargo run -p invariant-path-cli` directly instead.

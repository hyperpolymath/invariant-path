<!--
SPDX-License-Identifier: CC-BY-SA-4.0
SPDX-FileCopyrightText: 2026 Jonathan D.A. Jewell (hyperpolymath) <j.d.a.jewell@open.ac.uk>
-->

# Debt register — invariant-path

**Audited 2026-08-05** by reading the tree and querying the GitHub API. This is
the *live* register: findings stay until closed, and each carries the evidence
that would prove it fixed.

Supersedes `docs/tech-debt-2026-05-26.md`, whose "RESOLVED" banner overstates
the proof item — see [P1](#p1).

| | Dimension | Open | Worst |
|---|---|---|---|
| A | Architecture — the three layers | 3 | **HIGH** |
| C | CI/CD | 2 | **CRITICAL** |
| L | Licence | 3 | MEDIUM |
| D | Docs | 3 | MEDIUM |
| P | Proof | 2 | MEDIUM |
| K | Code | 1 | LOW |

---

## Architecture debt

This is where the substantial work is, and where the substantial gaps are.
ADR-0001 settled Layer 3's charter and name; it did not build anything, and it
was explicit about what remains.

<a id="a1"></a>
### A1 — Layer 1 has no substrate anywhere. `HIGH`

A survey of `_TYPES _SET` and `_PROVER-SOLVER _SET` (recorded in ADR-0001)
found **no syntax of claims, premises, steps or evidence in any repository**,
and **no ordering on evidence events of any kind**. All three sub-layers —
well-formedness, continuity, temporality — must be built from scratch.

Two names promise otherwise and do not deliver: `choreographic-types` has never
contained a line of source on any branch, and typell's "Level 9 Temporal
safety" passes iff a named field is present, examining no event and no order.

**Closed when:** a claim syntax and an evidence-event ordering exist, with the
diode enforced on the latter.

### A2 — Layer 3's equivalence side does not exist. `HIGH`

`echo-types` supplies roughly a quarter of the layer, and only the
*obstruction* half: the fibre `Echo f y = Σ A (λ x → f x ≡ y)`, an invariant
separating two members of one fibre, and "identified members count once".

There is **no move relation, no quotient, and no normal form anywhere** — which
is the entire equivalence-certificate side. And the one piece most needed is
`echo-types`' own unclosed debt: propositional truncation is *postulated* under
`--safe` and constructed only in a `--cubical` island that Agda's flag rules
forbid the main cone from importing. "Merely inhabited versus how many
witnesses" is exactly what the doubling attack turns on.

Two candidate substrates were investigated and **refuted with evidence** —
QuandleDB and absolute-zero's CNO — so this is not a matter of wiring something
up. Do not reopen either without new evidence; both refutations are in
ADR-0001.

**Recorded ruling:** mechanise in **Agda `--cubical`**, not Idris 2. The core
construction is a quotient by admissible moves, and Idris 2's QTT cannot host
quotients or higher inductive types.

**Closed when:** a move relation and a quotient exist, and an equivalence
certificate can be produced and checked for a worked pair.

### A3 — Layer 2's vocabulary does not match its implementation. `MEDIUM`

The terms in circulation — strength, consistency, coherence,
resemblance-warrant-on-merge, p-residue, p-sufficiency — are not the terms in
code. Only **p-sufficiency** is real (`floor(U) ⊑ acc(v)`). **`p-residue` does
not exist in any repository in the estate.**

What is built (`trope-checker`) is a six-coordinate `Grade`, nine `p-*`
effects, a `Floor`, and a witness-carrying `Verdict`.

This matters beyond tidiness: part of the ADR-0001 argument read p-residue as
affine resource accounting — a term that could not be grounded — so the
reachability result should be re-derived against the actual grade algebra.
`trope-particularity-workbench/tests/check-vocabulary.sh` hard-fails on any
vocabulary entry outside the nine effects, so introducing these terms as if
established would break a live gate.

Left deliberately unreconciled in ADR-0001 rather than quietly harmonised.

**Closed when:** a ruling records whether the six terms are a new Layer 2
design superseding the grade algebra, an informal gloss of it, or drift.

---

## CI/CD debt

### C1 — Every workflow fails to start; `main` has never been green. `CRITICAL`

There is **no `.github/actions-lock.json`**, and this repository is subject to
estate-wide Actions lockfile enforcement. Every run terminates as
`startup_failure` before a single step executes.

Measured 2026-08-05 across all branches:

```
main                        6/6 startup_failure
docs/state-the-idea         6/6 startup_failure
docs/adr-equivalence-layer  6/6 startup_failure
```

Seventeen workflow files exist. None has ever run to completion.

This is the most consequential item in the register, because it makes every
other gate decorative: CodeQL, secret-scanner, governance and Rust-CI cannot
fail, since they cannot start.

> **Diagnosis trap.** `gh pr checks` shows *nothing wrong* — a parse-rejected
> workflow produces no check run at all, so a PR looks unchecked rather than
> broken. Use `gh run list --branch <b>`; a `?status=failure` query also
> **excludes** `startup_failure`.

**Fix:** add `.github/actions-lock.json`, hand-author the reusable-caller
entries (the lock generator skips callers), and re-pin. The chain is proven
elsewhere in the estate.

**Closed when:** `gh run list --branch main` shows zero `startup_failure`.

### C2 — No proof gate. `MEDIUM`

No workflow invokes Agda, and none runs `scripts/verify-same-cube.sh`. The CI
half of [P1](#p1) and [P2](#p2).

**Closed when:** a workflow compiles `proofs/SameCube.agda` and runs the
verifier, and both can fail the build.

---

## Licence debt

### L1 — GitHub reports **no licence** for this repository. `MEDIUM`

`gh repo view --json licenseInfo` returns `null`, despite a 373-line MPL-2.0
`LICENSE` at the root and `license = "MPL-2.0"` in `Cargo.toml`. The repository
shows as unlicensed to anyone browsing it and to tooling reading the API.

Cause: `LICENSE` deviates from canonical MPL-2.0 in two places — a **trailing
space** at line 38, and `http://` rather than `https://` at line 360.
`LICENSES/MPL-2.0.txt` is the clean copy.

Fixed in this change by replacing `LICENSE` with the canonical text; detection
is recomputed on push, so the API result must be re-checked after merge.

**Closed when:** `gh repo view --json licenseInfo` reports `MPL-2.0`.

### L2 — SPDX headers are not on line 1. `MEDIUM`

Twelve of thirteen root-level documents open with `<!--` and carry the SPDX
identifier on line 2 or 3. The estate linter greps `head -1` only, so it reads
these as **missing**.

> **Do not fix by adding headers.** A blind sweep of this shape previously
> duplicated 24 headers and mis-licensed 3 files across the estate. The
> identifier must be *moved*, not imposed, and only after grepping the whole
> file to confirm one is not already present.

**Closed when:** every SPDX identifier is on line 1, with no duplicates and no
licence changed.

### L3 — `reuse lint` has one uncovered file. `LOW`

Pre-existing; the count has been stable at 1. Worth resolving alongside L2,
since both touch the same headers.

---

## Documentation debt

### D1 — Root `ARCHITECTURE.md` was template boilerplate. `MEDIUM` — fixed here

It described `src/`, `tests/`, `config/` and `README.adoc`. This repository has
`crates/`, `profiles/`, `schemas/`, `proofs/` and `README.md`; `tests/` and
`config/` do not exist. It also duplicated the real `docs/ARCHITECTURE.md`, and
said nothing whatever about the three-layer architecture — the repository's
principal subject.

Rewritten in this change against the actual tree, with the layers foregrounded.

### D2 — The README described a superseded verification method. `MEDIUM` — fixed here

`README.md` stated that every face's lowering "must normalise to the same
canonical text". The verifier was upgraded to compile each face to typed-wasm
and compare `sha256` hashes, using the text diff only as a diagnostic.

The README had drifted from the tool **in exactly the way the tool exists to
catch** — a claim that held on both sides of a transition while no longer
meaning the same thing.

### D3 — The wiki was a default stub, and undocumented directories. `LOW` — fixed here

`Home.md` read `Welcome to the invariant-path wiki!` at one commit. Separately,
`src/ui/` (an AffineScript GUI sketch, in no build) and `proofs/` appeared in
no document.

---

## Proof debt

<a id="p1"></a>
### P1 — `SameCube.agda` is real but ungated. `MEDIUM`

The proof is sound work: `--safe`, stdlib-free, `Trace` as a module parameter,
**zero postulates and zero holes**. Better than much of the estate.

But nothing compiles it. `docs/tech-debt-2026-05-26.md` marks proof debt
`RESOLVED` on the grounds that the file exists. Existing is not being checked:
if the proof broke tomorrow, no gate would notice.

**Correction to the record:** proof debt is *half* closed — the artefact
exists, the verification does not.

It also covers a fragment by design: observational identity holds **only for
unit-returning actions**, since non-unit tails have different result types.
That boundary is stated in the proof and in `profiles/faces.md`; it is recorded
here so the gap is not mistaken for a passing grade.

**Closed when:** CI compiles it and a deliberately broken variant fails.

<a id="p2"></a>
### P2 — The same-cube verifier exits 0 when it cannot run. `MEDIUM`

`scripts/verify-same-cube.sh:59-62` prints `SKIP` and **exits 0** when no
`affinescript` binary is reachable. Defensible locally — there is nothing to
check without a compiler — but it is the estate's classic fake-gate shape, and
in CI it would report success having verified nothing.

**Fix:** keep the local SKIP; make a missing compiler a hard failure when
`CI=true`, so the exit code distinguishes "verified" from "could not look".

**Closed when:** the script exits non-zero on a missing compiler under `CI`.

---

## Code debt

### K1 — Thin test coverage relative to claims. `LOW`

Twenty-one `#[test]` functions across the workspace, and **zero** `TODO`,
`FIXME`, `todo!()` or `unimplemented!()` — the source is clean, which is worth
saying plainly.

The gap is proportional, not absolute: the classifier's behaviour is defined by
roughly fifteen term lists and their pairwise interactions, a larger surface
than twenty-one tests can pin. Property tests exist (Phase A shipped proptest)
but the loss-tag matrix is not systematically covered.

Explicitly **not** worth addressing before [C1](#c1--every-workflow-fails-to-start-main-has-never-been-green-critical)
— tests that cannot run in CI buy nothing.

---

## Not debt

Recorded so they are not re-raised:

- **The `faces` profile vendors its corpus** and is named for a claim rather
  than a target. Real, but already tracked as **issue #54** with an agreed fix.
- **The `greet` corpus splits into two wasm classes.** Expected: byte-identical
  wasm is *stricter* than observational equivalence, and the split is over
  statement-versus-tail-expression lowering. Documented in `profiles/faces.md`.
- **The classifier is heuristic and accepts false negatives.** A deliberate
  decision (`DESIGN.md`, 2026-04-10).
- **The three layers are unimplemented.** Tracked above as A1–A3 rather than as
  a defect: ADR-0001 is a decision record about an architecture and says so.

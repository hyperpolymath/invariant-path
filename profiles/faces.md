<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# Profile: faces

**Target corpus:** the AffineScript face family —
`affinescript/examples/faces/`, the `examples/same-cube/` corpus in this
repo, and the face brand-surface repos (`rattlescript`, `jaffascript`,
`pseudoscript`, `lucidscript`, `cafescripto`).
**Mode:** invariant-grounding (behavioural equality, not doc-claims)
**Purpose:** Keep the load-bearing AffineScript claim — *"different faces,
same cube"* — grounded in the actual behaviour of the face transformers.

## Why this profile exists

AffineScript is one canonical language (the "cube"). Each *face* (ADR-010)
is an alternative surface syntax — RattleScript (Python), JaffaScript
(JS/TS), PseudoScript (pseudocode), LucidScript (PureScript/Haskell),
CafeScripto (CoffeeScript) — lowered to canonical AffineScript by a pure
text transformer (`lib/<name>_face.ml`) and previewed with `preview-*`.

The whole architecture rests on one invariant:

> For a given program, **every face lowers to the same canonical cube**
> (modulo whitespace and comment placement), and therefore to the same
> typed-wasm output.

That is a *claim*. Like any claim, it can rot: a transformer edit can
silently make one face diverge while the others still pass their own
round-trip tests. Per-face snapshot tests (`affinescript/tests/faces/`)
catch *drift within a face* but not *divergence between faces* — they
never compare face A's cube against face B's. This profile grounds the
cross-face equality directly: it is a claim-path debugger for the
same-cube invariant, locating *which* face breaks the cube and *where*.

## The claim path

```
face source (greet/rattle.affine, …)         ← evidence / specification
        │  preview-<face>                     ← transition (transformer T)
        ▼
typed-wasm module (compile --face)            ← the cube itself
        │  sha256-equality vs canonical wasm  ← grounding check
        ▼
"same cube" holds for this program            ← conclusion (Grounded | Ungrounded)
```

## Invocation

```sh
# Ground the bundled same-cube corpus (needs an `affinescript` binary on
# PATH, or a dune build of it — see the script's resolver).
scripts/verify-same-cube.sh examples/same-cube/greet \
  --out .machine_readable/audits/same-cube.jsonl

# Any directory of sibling face files for ONE program works:
scripts/verify-same-cube.sh path/to/<program>-faces/
```

A face brand-surface repo grounds its own examples by pointing the shared
workspace at its corpus (the `tools/invariant-path/` hook), e.g.:

```sh
just invariant-path same-cube examples/
```

## What it catches

- A face whose lowering compiles to a **different typed-wasm module** than the
  canonical reference (the cross-face equality break the per-face snapshots
  miss). Comparing wasm — the cube itself — also avoids text-only
  false-positives (e.g. tail-expression vs statement lowering, observationally
  identical but textually different).
- A face example that fails to parse or compile (round-trip / build break).
- A corpus where one face silently encodes a *different* program.

When faces split into more than one wasm class, the `preview-*` text diff is
printed per divergence so you can see *where* the lowering parts ways.

## Limits

- **Byte-identical** wasm is stricter than **observational** equivalence: two
  faces can print the same thing and return the same value yet emit different
  wasm. The grounded `greet` result is exactly this — rattle/pseudo/lucid vs
  canonical/jaffa/cafe split into two classes over a trailing-statement
  lowering choice. The tool reports the wasm *classes* and leaves "are these
  classes observationally equal?" to the reader; it does not execute the
  modules to compare runtime output.
- It grounds the corpora it is given; it does not enumerate all programs.
- Effect-handler lowering soundness (tracked in affinescript #555); a
  face that exercises `handle` may converge in text yet diverge at runtime.

<!-- SPDX-License-Identifier: MPL-2.0 -->

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
canonical text (normalised)                   ← intermediate target
        │  byte-equality vs canonical.affine  ← grounding check
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

## What v1 catches

- A face whose `preview-*` lowering no longer normalises to the canonical
  cube (the cross-face equality break the per-face snapshots miss).
- A face example that fails to parse after lowering (round-trip break).
- A corpus where one face silently encodes a *different* program (e.g. a
  changed string literal or dropped statement).

## What v1 does NOT catch (yet)

- *Semantic* divergence below the canonical-text level — two different
  canonical texts that nevertheless typecheck to the same cube. v1
  compares normalised canonical text, not ASTs. (AST-level equality is
  the planned v2 grounding, once the compiler exposes a stable AST dump.)
- typed-wasm equality. The chain "same canonical ⇒ same wasm" is asserted
  by the compiler, not re-checked here.
- Effect-handler lowering soundness (tracked in affinescript #555); a
  face that exercises `handle` may converge in text yet diverge at runtime.

<!-- SPDX-License-Identifier: MPL-2.0 -->

# Proofs

Machine-checked proofs supporting the `faces` profile and the same-cube
verifier.

## `SameCube.agda`

The lemma underneath the same-cube grounding. Running `verify-same-cube.sh`
against a real affinescript build splits the `greet` corpus into two wasm
classes — `{ a; }` (statement; the canonical/jaffa/cafe class) vs `{ a }`
(tail-expression; the rattle/pseudo/lucid class). This module proves that for a
**unit-returning** effectful action those two lowerings are *observationally
identical* (same effect trace, same unit return), so the split denotes the
**same cube**. It also pins the boundary: for a non-unit tail the two lowerings
have different *result types*, so a value-returning corpus would genuinely
diverge — which is the precise formal reason the equivalence is unit-tail-only.

Check it:

```sh
agda --safe proofs/SameCube.agda      # or:  just proofs
```

`--safe` rules out postulates and other escape hatches, so this is a real
constructive proof, not an assertion. Agda 2.6.x; self-contained (no stdlib),
with `Trace` abstracted as a module parameter.

## Scope

This proves the *observational* equivalence of the two lowering styles for the
unit case. The stronger obligation — full **transformer semantics-preservation**
(every `lib/<face>_face.ml` transform preserves the typed-wasm denotation for
all programs) — remains future work; it needs the affinescript AST + wasm
semantics formalised, not just this block-lowering fragment.

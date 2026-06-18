<!-- SPDX-License-Identifier: MPL-2.0 -->

# Same-cube corpus

Each subdirectory here is **one program written in every AffineScript face**.
The files are deliberately the *same* program (identical strings, identical
structure) — not the demonstrative, different-string examples in
`affinescript/examples/faces/`. That is the whole point: if they are the same
program, every face should lower to the **same cube**.

```
greet/
  canonical.affine   face: canonical      (the reference cube)
  rattle.affine      face: rattlescript   -> preview-python
  jaffa.affine       face: jaffascript    -> preview-js
  pseudo.affine      face: pseudoscript   -> preview-pseudocode
  lucid.affine       face: lucidscript    -> preview-lucid
  cafe.affine        face: cafescripto    -> preview-cafe
```

## Ground the invariant

```sh
scripts/verify-same-cube.sh examples/same-cube/greet \
  --out .machine_readable/audits/same-cube.jsonl
```

The verifier detects each file's face from its `face:` pragma, **compiles it to
typed-wasm** (`compile --face <face>`), and sha256-compares the modules — the
wasm *is* the cube, so byte-identical wasm is the rigorous bar. For any face
outside the canonical class it also prints the `preview-*` text diff as a
diagnostic of *where* the lowering diverges. Output is a per-face table, the
wasm equivalence classes, and invariant-path claim records (`Grounded` /
`Ungrounded`). Needs an `affinescript` binary (PATH, `--affinescript`,
`AFFINESCRIPT`, or `../affinescript/_build`); without one it SKIPs.

This is a **claim-path debugger**, not a rubber stamp: a `DIFF` is the tool
locating which face leaves the cube and showing exactly where.

## Grounded result — `greet` (2026-06-18, affinescript @ main)

Run against a freshly-built compiler, the six faces compile to **two** distinct
wasm modules:

| wasm class | faces |
|---|---|
| `56c454be…` | canonical, **jaffa**, **cafe** |
| `2ff63dd1…` | **rattle**, **pseudo**, **lucid** |

The split is **not** a transformer crash — all six compile, type-check, and
print the same string. It is a real, characterised divergence in *lowering
style*: rattle/pseudo/lucid render the trailing call as a **tail expression**
(`fn main() … { println(x) }`), while canonical/jaffa/cafe keep it as a
**statement** (`{ println(x); }`). For `println : … -> ()` both return unit, so
the programs are **observationally identical** — but the emitted wasm is not
byte-identical.

So *"different faces, same cube"* is true **observationally** but **false at the
byte-wasm level** for this corpus: the face transformers do not currently agree
on trailing-statement lowering. Tracked as an affinescript transformer-
consistency item. (lucid additionally emits a `module Greet;` decl and drops
the `-{IO}->` annotation in its preview text; neither changes the wasm class.)

## Adding a program

Add a sibling directory (e.g. `counter/`) with one file per face, each carrying
the right `face:` pragma and encoding the identical program. Keep to surface
features the transformers handle today (see the "Known transformer gaps" table
in `affinescript/examples/faces/README.adoc`); divergences show up as extra
wasm classes — useful signal, not noise.

<!-- SPDX-License-Identifier: MPL-2.0 -->

# Same-cube corpus

Each subdirectory here is **one program written in every AffineScript face**.
The files are deliberately the *same* program (identical strings, identical
structure) — not the demonstrative, different-string examples in
`affinescript/examples/faces/`. That is the whole point: if they are the same
program, every face must lower to the **same canonical cube**.

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

The verifier detects each file's face from its `face:` pragma, runs the
matching `preview-*` transformer, normalises the result (modulo comments and
whitespace), and compares it to `canonical.affine`. Output is a per-face
table plus invariant-path claim records (`Grounded` / `Ungrounded`).

This is a **claim-path debugger**, not a rubber stamp: a `DIFF` line does not
mean the tool failed — it means the tool located the face that diverges from
the cube, and prints exactly where. Grounding requires an `affinescript`
binary (PATH, `--affinescript`, `AFFINESCRIPT`, or `../affinescript/_build`);
without one the verifier SKIPs, because the invariant is grounded in CI where
the compiler is built.

## Adding a program

Add a new sibling directory (e.g. `counter/`) with one file per face you want
to cover, each carrying the right `face:` pragma and encoding the identical
program. Keep to surface features the transformers handle today (see the
"Known transformer gaps" table in `affinescript/examples/faces/README.adoc`);
a face that uses an unsupported construct will show up as a `DIFF`, which is
useful signal, not noise.

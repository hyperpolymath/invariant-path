<!--
SPDX-License-Identifier: MPL-2.0
Copyright (c) Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk>
-->
# Profile: pmpl

**Target corpus:** `~/Documents/hyperpolymath-repos/palimpsest-license/`
**Mode:** doc-claims (factual grounding)
**Purpose:** Keep the Palimpsest Public License (PMPL) text grounded in
the file system claims it makes about itself, the Palimpsest covenant,
the standard SPDX header form, and the legal-fallback chain to MPL-2.0.

## Why this profile exists

The PMPL text and its accompanying covenant are *normative* documents:
they make promises about how the licence behaves, where the canonical
text lives, what SPDX identifier to use, and what the legal fallback is.
If the text references `LICENSE-MPL-2.0` and that file isn't in the
repo, the licence is making a false structural claim about its own
fallback. That's a worse kind of bullshit than the ordinary kind:
*licence-text bullshit*. So we ground it.

## Invocation

```sh
cargo run --quiet -p invariant-path-cli -- doc-claims \
  --root ~/Documents/hyperpolymath-repos/palimpsest-license \
  --out  ~/Documents/hyperpolymath-repos/palimpsest-license/.machine_readable/audits/doc-claims.jsonl
```

## What v1 catches

- `LICENSE`, `LICENSE-MPL-2.0`, `PALIMPSEST-COVENANT.md`, `NOTICE` etc.
  referenced from the licence text — must actually exist in the repo.
- Backtick-quoted SPDX-License-Identifier examples — verifies the
  identifier strings are syntactically the ones the licence policy
  document expects.
- Cross-references to estate-wide canonical paths
  (`standards/LICENCE-POLICY.adoc`, etc.) — flagged as cross-repo by
  the standards-docs profile workflow.

## What v1 does NOT catch (yet)

- *Semantic* claims about clauses ("section 3.2 grants X"). Grounding
  those needs a clause-level parser.
- Translation-equivalence claims ("the French version says the same
  thing"). Out of scope.
- Re-licensing chains across third-party dependencies. That's the job
  of the upcoming PLASMA repo-sweeper, not invariant-path.

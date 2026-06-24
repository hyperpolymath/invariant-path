<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# Profile: standards-docs

**Target corpus:** `~/Documents/hyperpolymath-repos/standards/`
**Mode:** doc-claims (factual grounding)
**Purpose:** Keep the hyperpolymath standards corpus honest. Every file
path the standards documents reference should actually exist; every
A2ML key claim should resolve; every "command X returns clean" should be
verifiable from a CI artefact.

## Why this profile exists

The standards repo is the *single source of truth* for what the
hyperpolymath estate claims about itself (CRG, TRG, RSR, Immaculate
Guide, Palimpsest, A2ML format, k9-svc, etc.). When a standards document
says "the canonical proof suite lives at `templates/CANONICAL-PROOF-SUITE.adoc`",
that file had better exist. When it doesn't, the standard is making a
factual claim about a file system state that isn't true — i.e. it is
*bullshitting* about itself.

This profile is the panic-attacker for that prose.

## Invocation

From the invariant-path workspace:

```sh
cargo run --quiet -p invariant-path-cli -- doc-claims \
  --root ~/Documents/hyperpolymath-repos/standards \
  --out  ~/Documents/hyperpolymath-repos/standards/.machine_readable/audits/doc-claims.jsonl
```

Exit code 2 on any ungrounded claim. Wire this into the standards repo's
CI just like `panic-attack assail`.

## Known limitations (v1)

- **Single-root scan.** A claim like `` `0-AI-MANIFEST.a2ml` `` in
  `standards/foo.adoc` is not "the AI manifest of the *standards* repo"
  — it's a generic statement about the per-repo manifest convention.
  v1 will mark it as ungrounded relative to `standards/`. Multi-root /
  cross-repo grounding is on the roadmap.
- **Recogniser surface.** v1 understands backtick-quoted paths,
  A2ML-presence assertions, and command-clean claims. It does not yet
  understand version-pin claims, badge claims, or "X is at Y line N"
  positional references. Those are easy adds and should follow real
  usage.
- **No A2ML parser yet.** A2ML key checks are lexical only. Replace with
  a real parser when `a2ml-rs` is available.
- **No `--allow-exec` resolver.** Command-clean claims are reported as
  Unknown by default. CI artefact-based resolution is planned (read the
  most recent recorded run from `.machine_readable/audits/`).

## Triage workflow

A first run on a real corpus will return a *lot* of ungrounded claims.
Triage them in this order:

1. **Drift.** The claim is false because the file got renamed or moved.
   Fix the standard.
2. **Vaporware.** The claim names a file that was promised but never
   created. Either create the file or remove the claim.
3. **Cross-repo reference.** The claim is correct but lives in a
   different repo. Note for v2 (multi-root mode); for now, accept the
   ungrounded result with an inline `<!-- ipdc:cross-repo -->` marker
   (suppression syntax also v2).
4. **Genuine bug in the recogniser.** The claim isn't really a claim.
   File an issue against invariant-path with the offending sentence.

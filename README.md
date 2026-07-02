<!--
SPDX-License-Identifier: CC-BY-SA-4.0
SPDX-FileCopyrightText: 2025-2026 Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk>
-->

[![OpenSSF Best Practices](https://img.shields.io/badge/OpenSSF-Best_Practices-green?logo=opensourcesecurity)](https://www.bestpractices.dev/en/projects/new?repo_url=https://github.com/hyperpolymath/invariant-path)
[![License: MPL-2.0](https://img.shields.io/badge/License-MPL--2.0-blue.svg)](LICENSE) <embed
src="https://api.thegreenwebfoundation.org/greencheckimage/github.com"
data-link="https://www.thegreenwebfoundation.org/green-web-check/?url=github.com" />

Invariant Path is a repo-native semantic overlay MVP for tracing how
claims move from source evidence/specification to target conclusions.

It is a claim-path debugger, not a truth engine.

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

- `examples/` — seeded and realistic examples (incl.
  `examples/same-cube/` — the AffineScript faces corpus)

- `profiles/` — profile notes for `echidna`, `panll`, `hypatia`, `pmpl`,
  and `faces` (AffineScript "different faces, same cube")

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

<!--
SPDX-License-Identifier: CC-BY-SA-4.0
SPDX-FileCopyrightText: 2025-2026 Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk>
-->

[![OpenSSF Best Practices](https://img.shields.io/badge/OpenSSF-Best_Practices-green?logo=opensourcesecurity)](https://www.bestpractices.dev/en/projects/new?repo_url=https://github.com/hyperpolymath/invariant-path)
[![License: PMPL-1.0](https://img.shields.io/badge/License-MPL--2.0-blue.svg)](https://github.com/hyperpolymath/palimpsest-license) <embed
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
cargo run -p invariant-path-cli -- tui --file ./README.md --artifact-uri repo://README.md
```

# Launcher Integration

The repository includes an E-Grade compliant launcher and desktop entry:

- Launcher script:
  `/var/mnt/eclipse/repos/.desktop-tools/invariant-path-launcher.sh`

- Repo-root command: `./invariant-path`

- Desktop template: `desktop/invariant-path.desktop`

- Installer: `scripts/install-desktop.sh`

Install Start Menu + Desktop shortcuts:

```bash
./scripts/install-desktop.sh
```

## TUI Mode

The launcher supports an interactive Terminal User Interface:

```bash
# Launch TUI (automatic fallback to scan if terminal not available)
./invariant-path-launcher --tui

# Direct CLI TUI access
cargo run -p invariant-path-cli -- tui --file README.adoc
```

The TUI provides: \* Interactive navigation through claim path
suggestions \* Keyboard controls (↑/↓ to navigate, q to quit) \* Visual
highlighting of selected items \* Automatic fallback to CLI mode in
non-interactive environments

See [TUI Guide](docs/TUI-GUIDE.md) for comprehensive documentation.

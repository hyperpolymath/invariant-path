<!--
SPDX-License-Identifier: MPL-2.0
Copyright (c) Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk>
-->
# Invariant Path MVP Design Log

## 2026-04-10 — Initial Placement

- Decision: build a standalone monorepo-style workspace at `/var/mnt/eclipse/repos/invariant-path`.
- Tradeoff: wrappers in `echidna`, `panll`, and `hypatia` call the shared CLI instead of duplicating logic.
- Reasoning: this keeps heuristics inspectable and consistent while still enabling repo-specific usage patterns.

## 2026-04-10 — Heuristic Philosophy

- Decision: favor strict trigger-based extraction with explicit transition markers.
- Tradeoff: we intentionally accept false negatives to reduce annotation noise.
- Reasoning: contributors can always add/edit annotations manually; noisy auto-suggestions hurt trust.

## 2026-04-10 — Storage Model

- Decision: use append-friendly JSONL (`annotations.jsonl`) with upsert-on-write semantics.
- Tradeoff: not ideal for very large datasets, but easy to inspect and diff in git.
- Reasoning: MVP prioritizes transparency and editability over throughput.

## 2026-04-10 — Profile Injection via Wrappers

- Decision: each target repo (`echidna`, `panll`, `hypatia`) gets a tiny shell wrapper that injects a default `--profile` for `scan`.
- Tradeoff: wrappers rely on local path `../invariant-path` and `cargo run`, not a globally installed binary.
- Reasoning: fastest path to real usage across multiple repos while preserving one shared implementation.

## 2026-04-10 — Status Semantics in CLI

- Decision: map core UX actions directly to commands: `accept`, `dismiss`, `clarify`, `add`, `update`, plus `overlay toggle`.
- Tradeoff: no rich side panel UI in MVP; interaction is CLI-first with JSON output.
- Reasoning: preserves complete editability and auditability while keeping implementation small.

## 2026-04-10 — Desktop and Start Menu Integration

- Decision: add a shared launcher at `/var/mnt/eclipse/repos/.desktop-tools/invariant-path-launcher.sh` plus an install script that writes `.desktop` entries to start menu and desktop locations.
- Tradeoff: launcher defaults to CLI-friendly `Terminal=true` because Invariant Path is currently a CLI-first tool.
- Reasoning: satisfies immediate usability from Start Menu, desktop, and repo-root without introducing an incomplete GUI overlay.

## 2026-04-10 — Repo-Root Invocation Standard

- Decision: expose `invariant-path` entry points in each integrated repo through `scripts/invariant-path.sh` and `just invariant-path ...`.
- Tradeoff: requires shared workspace path `../invariant-path` to exist.
- Reasoning: keeps behavior consistent across repos while preserving a single source of truth for extractor/classifier logic.

## 2026-04-10 — Desktop Installer Idempotence

- Decision: make `scripts/install-desktop.sh` remove a destination `.desktop` file if it exists but is not writable, then reinstall with deterministic modes (`444` in app menu, `555` on Desktop/Shortcuts).
- Tradeoff: replacement is explicit rather than in-place update, but content and permissions remain deterministic from template + destination policy.
- Reasoning: GNU `install` can fail on existing read-only targets; KDE/Plasma desktop launchers also need execute bits to avoid launch aborts.

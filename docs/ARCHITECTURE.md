<!--
SPDX-License-Identifier: MPL-2.0
Copyright (c) Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk>
-->
# Minimal Architecture Proposal

Invariant Path is split into four inspectable modules:

1. `extractor`
- Input: repository artifact text.
- Output: candidate claim transitions anchored by trigger phrases and spans.

2. `classifier`
- Input: extracted candidate transition.
- Output: invariant type, preserved/losses, break condition, and classification enum.

3. `annotations`
- Input: machine suggestions and manual edits.
- Output: persisted overlay records with visibility and status controls.

4. `ui` (CLI first)
- Input: user command intents (`scan`, `accept`, `dismiss`, `clarify`, `add`, `update`).
- Output: JSON/human-readable overlays; optional persisted suggestions.

## Data Flow

`artifact text -> extractor -> classifier -> suggestion annotations -> user edits -> stored overlay`

## Integration Strategy

- Shared core workspace: `invariant-path`.
- Repo wrappers:
  - `echidna/scripts/invariant-path.sh`
  - `panll/scripts/invariant-path.sh`
  - `hypatia/scripts/invariant-path.sh`
- Profiles allow domain-specific defaults without changing core extraction/classification logic.

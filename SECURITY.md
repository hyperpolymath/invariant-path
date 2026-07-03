<!--
SPDX-License-Identifier: CC-BY-SA-4.0
SPDX-FileCopyrightText: 2026 Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk>
-->

# Security Policy

## Reporting a vulnerability

Email <j.d.a.jewell@open.ac.uk> with a description and reproduction steps.
Please do not open public issues for security reports. You should receive an
acknowledgement within 14 days.

## Scope

Invariant Path is a local CLI that reads files you point it at and writes
JSONL annotations under the working directory. It makes no network calls.
The most security-relevant surfaces are:

- Path handling in `doc-claims` grounding (`crates/invariant-path-core/src/doc_claims.rs`)
- JSONL parsing in the annotation store (`crates/invariant-path-core/src/annotations.rs`)

## Supported versions

Only the latest release on `main` is supported.

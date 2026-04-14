# Extending Invariant Path

## Add a New Invariant Type

1. Edit `crates/invariant-path-core/src/model.rs` and add the enum variant to `InvariantType`.
2. Update `infer_invariant_type` in `crates/invariant-path-core/src/classifier.rs` with keyword rules.
3. Add test cases in `crates/invariant-path-core/src/lib.rs` tests.
4. If persistence schema should enforce it, update `schemas/annotation.schema.json` enum.

## Add or Tune Heuristics

1. Edit trigger phrases in `crates/invariant-path-core/src/extractor.rs`.
2. Edit high-value jump maps in `crates/invariant-path-core/src/classifier.rs`.
3. Keep heuristics explicit and local; avoid hidden statistical/ML behavior in MVP.
4. Add/adjust tests for both positive and negative examples.

## Profile-Specific Behavior

Profiles are currently selected by CLI flag (`--profile echidna|panll|hypatia`).

To add one:

1. Update `ProfilePreset` in `crates/invariant-path-cli/src/main.rs`.
2. Add default artifact globs and visibility policy.
3. Add a smoke test scenario in `examples/`.

## Design Constraints

- Every automatic judgment must remain user-editable.
- Favor false negatives over noisy false positives.
- Keep storage inspectable and git-friendly.

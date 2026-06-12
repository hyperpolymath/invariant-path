// SPDX-License-Identifier: MPL-2.0
// SPDX-FileCopyrightText: 2026 Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk>
//
// CI-covered companion to examples/echo_structured_loss.rs.
//
// Pins the echo-types correspondence as a test: `classify_candidate` is a
// non-injective classifier, and the fiber (echo) over a classification retains
// the per-candidate structured loss that the bare `Classification` discards.
// See docs/ECHO-TYPES.md.

use invariant_path_core::classifier::classify_candidate;
use invariant_path_core::model::{ClaimCandidate, Classification, Span};

fn candidate(source: &str, target: &str, trigger: &str) -> ClaimCandidate {
    ClaimCandidate {
        span: Span { start: 0, end: 0 },
        source_text: source.to_string(),
        target_text: target.to_string(),
        trigger: trigger.to_string(),
        path_description: format!("{source} -> {target}"),
    }
}

/// Two distinct claim-paths collapse to one classification: the classifier is
/// non-injective, so the classification alone cannot recover the path.
#[test]
fn classification_is_non_injective() {
    let c1 = candidate("benchmark accuracy", "general capability", "therefore");
    let c2 = candidate("the theorem proves", "production guarantee", "implies");

    let o1 = classify_candidate(&c1);
    let o2 = classify_candidate(&c2);

    assert_ne!(c1, c2);
    assert_eq!(o1.classification, Classification::Overextended);
    assert_eq!(o2.classification, Classification::Overextended);
}

/// The echo retains what the classification drops: distinct candidates over the
/// same classification carry distinct, retained `losses` (the structured loss).
#[test]
fn echo_retains_structured_loss() {
    let c1 = candidate("benchmark accuracy", "general capability", "therefore");
    let c2 = candidate("the theorem proves", "production guarantee", "implies");

    let o1 = classify_candidate(&c1);
    let o2 = classify_candidate(&c2);

    // same fiber (classification) …
    assert_eq!(o1.classification, o2.classification);
    // … but distinct, retained structured loss — the echo keeps it.
    assert_ne!(o1.losses, o2.losses);
    assert!(!o1.losses.is_empty() && !o2.losses.is_empty());
}

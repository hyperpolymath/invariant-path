// SPDX-License-Identifier: MPL-2.0
// SPDX-FileCopyrightText: 2026 Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk>
//
// Worked echo-types example: classification as fiber-based structured loss.
//
// Invariant Path's classifier
//     classify_candidate : ClaimCandidate -> ClassificationOutcome
// is a NON-INJECTIVE map: many distinct claim-paths collapse to the same
// `Classification`. hyperpolymath/echo-types is the mechanised theory of
// exactly this collapse — the fiber
//     Echo f y  :=  Σ (x : A) , (f x ≡ y)
// the proof-relevant record of *which* inputs map to y.
//
// This example materialises one echo: the fiber of `Overextended`. The bare
// `Classification::Overextended` forgets which path produced it; Invariant
// Path RETAINS the full candidate together with its `losses` — and that
// retained structured loss IS the echo. Invariant Path is, in echo-types
// terms, the runtime that keeps echoes instead of discarding them.
//
// Run:  cargo run -p invariant-path-core --example echo_structured_loss
// See:  docs/ECHO-TYPES.md  and the Agda companion
//       nextgen-typing/verification/proofs/agda/EchoTyping.agda

use invariant_path_core::classifier::classify_candidate;
use invariant_path_core::model::{ClaimCandidate, Classification, ClassificationOutcome, Span};

/// Build a minimal candidate (spans are irrelevant to classification here).
fn candidate(source: &str, target: &str, trigger: &str) -> ClaimCandidate {
    ClaimCandidate {
        span: Span { start: 0, end: 0 },
        source_text: source.to_string(),
        target_text: target.to_string(),
        trigger: trigger.to_string(),
        path_description: format!("{source}  ⟶  {target}"),
    }
}

fn main() {
    // Two genuinely different claim-paths …
    let c1 = candidate("benchmark accuracy", "general capability", "therefore");
    let c2 = candidate("the theorem proves", "production guarantee", "implies");

    let o1 = classify_candidate(&c1);
    let o2 = classify_candidate(&c2);

    // … collapse to the SAME classification: the classifier is non-injective.
    assert_eq!(o1.classification, Classification::Overextended);
    assert_eq!(o2.classification, Classification::Overextended);
    assert_ne!(c1, c2, "the two claim-paths are genuinely distinct");

    // The echo (fiber over `Overextended`) is what Invariant Path RETAINS:
    // the distinct candidates plus their distinct structured losses, which the
    // bare `Classification` has forgotten.
    let fiber: Vec<(ClaimCandidate, ClassificationOutcome)> = vec![(c1, o1), (c2, o2)];

    println!("Echo(classify) over `Overextended` — the retained fiber:\n");
    for (cand, out) in &fiber {
        println!(
            "  • {:<32}  losses = {:?}",
            cand.path_description, out.losses
        );
    }

    // Distinct losses ⇒ the classification truly dropped information the echo keeps.
    assert_ne!(
        fiber[0].1.losses, fiber[1].1.losses,
        "each path's structured loss is distinct and retained"
    );

    println!(
        "\nNon-injective: 2 distinct paths → 1 classification.\n\
         The echo (candidate + losses) is the structured loss Invariant Path retains,\n\
         and `path_description` is the section that recovers the source the verdict forgot."
    );
}

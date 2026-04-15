// SPDX-License-Identifier: PMPL-1.0-or-later
#![forbid(unsafe_code)]

pub mod annotations;
pub mod classifier;
pub mod doc_claims;
pub mod extractor;
pub mod model;
pub mod pipeline;

#[cfg(test)]
mod tests {
    use crate::annotations::AnnotationStore;
    use crate::model::{Classification, InvariantType, Status, Visibility};
    use crate::pipeline::scan_artifact;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn proptest_scan_artifact_no_panic(s in "\\PC*") {
            let _ = scan_artifact("repo://proptest.md", &s, "proptester", Visibility::Private);
        }
    }

    fn first(text: &str) -> crate::model::Annotation {
        scan_artifact("repo://test.md", text, "tester", Visibility::Private)
            .into_iter()
            .next()
            .expect("expected at least one annotation")
    }

    #[test]
    fn benchmark_to_capability_is_overextended() {
        let ann = first("This benchmark proves the model can reason.");
        assert_eq!(ann.classification, Classification::Overextended);
        assert!(ann.losses.iter().any(|x| x == "task_transfer"));
    }

    #[test]
    fn theorem_to_guarantee_is_overextended() {
        let ann = first("This theorem guarantees production safety.");
        assert_eq!(ann.classification, Classification::Overextended);
        assert!(ann.losses.iter().any(|x| x == "implementation_gap"));
    }

    #[test]
    fn cost_transition_uses_resource_budget_invariant() {
        let ann = first("This cost reduction means transition is easy.");
        assert_eq!(ann.invariant_type, InvariantType::ResourceBudget);
    }

    #[test]
    fn local_to_universal_is_overextended() {
        let ann = first("This model result shows all workers are motivated by incentives.");
        assert_eq!(ann.classification, Classification::Overextended);
        assert!(ann.losses.iter().any(|x| x == "scope"));
    }

    #[test]
    fn uncertainty_to_normative_is_conflation() {
        let ann = first("This uncertainty means we should delay action.");
        assert_eq!(ann.classification, Classification::Conflation);
        assert_eq!(ann.invariant_type, InvariantType::NormativeBridge);
    }

    #[test]
    fn assumption_bound_generalization_is_incomplete() {
        let ann = first("Under assumptions, the local result implies all deployments are safe.");
        assert_eq!(ann.classification, Classification::Incomplete);
        assert!(ann.losses.iter().any(|x| x == "assumptions"));
    }

    #[test]
    fn data_to_policy_is_conflation() {
        let ann = first("The pilot data implies we should change policy immediately.");
        assert_eq!(ann.classification, Classification::Conflation);
        assert!(ann.losses.iter().any(|x| x == "normative_step"));
    }

    #[test]
    fn formal_corollary_path_is_valid() {
        let ann = first("The proof implies the corollary in the same formal system.");
        assert_eq!(ann.classification, Classification::ValidPath);
    }

    #[test]
    fn rhetorical_clearly_is_incomplete() {
        let ann = first("Clearly, this single test means every environment is secure.");
        assert_eq!(ann.classification, Classification::Incomplete);
        assert!(ann.losses.iter().any(|x| x == "missing_evidence"));
    }

    #[test]
    fn extractor_handles_multiple_sentences() {
        let text = "The benchmark is narrow. Therefore we should avoid broad deployment claims.";
        let ann = first(text);
        assert_eq!(ann.classification, Classification::Conflation);
        assert_eq!(ann.invariant_type, InvariantType::NormativeBridge);
    }

    #[test]
    fn store_round_trip_and_status_update() {
        let temp = std::env::temp_dir().join(format!("invariant-path-test-{}", std::process::id()));
        if temp.exists() {
            std::fs::remove_dir_all(&temp).expect("cleanup temp dir");
        }
        let store = AnnotationStore::new(&temp).expect("create store");

        let mut ann = first("This benchmark proves the model can reason.");
        ann.visibility = Visibility::SharedTeam;
        store.save(&ann).expect("save annotation");

        let fetched = store.get(&ann.id).expect("get annotation");
        assert_eq!(fetched.id, ann.id);

        let accepted = store
            .update_status(
                &ann.id,
                Status::Accepted,
                Some("accepted for now".to_string()),
            )
            .expect("update status");
        assert_eq!(accepted.status, Status::Accepted);

        let dismissed = store
            .dismiss_for_self(&ann.id, Some("dismiss local noise".to_string()))
            .expect("dismiss for self");
        assert_eq!(dismissed.status, Status::Dismissed);
        assert_eq!(dismissed.visibility, Visibility::Private);

        std::fs::remove_dir_all(&temp).expect("remove temp dir");
    }
}

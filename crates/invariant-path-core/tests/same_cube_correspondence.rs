// SPDX-License-Identifier: MPL-2.0
// Regression guards for the *scope* of the same-cube correspondence record.
// These tests check metadata consistency only. The Agda kernel, not these
// assertions, checks the theorem; there is no verified WASM refinement yet.

use serde_json::{json, Value};

const RECORD: &str = include_str!("../../../examples/same-cube/correspondence.json");
const PROOF: &str = include_str!("../../../proofs/SameCube.agda");

fn record() -> Value {
    serde_json::from_str(RECORD).expect("correspondence record must be valid JSON")
}

#[test]
fn unit_returning_claim_is_scoped_to_the_abstract_model() {
    let c = record();
    assert_eq!(c["format"], "invariant-path.same-cube-correspondence");
    assert_eq!(c["version"], 1);
    assert_eq!(c["relation"], "observational_equality");
    assert_eq!(c["domain"], "abstract_trace_and_result_model");
    assert_eq!(c["source"]["type"], "Obs unit");
    assert_eq!(c["target"]["type"], "Obs unit");
    assert_eq!(c["scope"]["return_kind"], "unit");
    assert_eq!(
        c["scope"]["observables"],
        json!(["effect_trace", "unit_result"])
    );
    assert_eq!(c["witness"]["kind"], "agda_kernel_proof");
    assert_eq!(c["witness"]["theorem"], "same-cube");
    assert_eq!(c["witness"]["scoped_contract"], "unitResultCorrespondence");
    // A source-reference guard, NOT a substitute for running Agda.
    assert!(PROOF.contains("same-cube : (a : Obs ⊤) → stmt a ≡ tail a"));
    assert!(PROOF.contains("unitResultCorrespondence = record { agrees = same-cube }"));
}

#[test]
fn non_unit_generalisation_has_a_concrete_counterexample() {
    let c = record();
    assert_eq!(
        c["excluded_generalisation"]["result_kind"],
        "non_unit_result_observed"
    );
    assert_eq!(
        c["excluded_generalisation"]["counterexample"],
        "nonUnitResultCounterexample"
    );
    assert!(PROOF.contains("nonUnitResultCounterexample :"));
    assert!(PROOF.contains("nonUnitResultCounterexample (trace , result) ()"));
    // No blanket claim that every non-unit comparison must differ:
    // the negative result uses an explicitly return-sensitive observer.
}

#[test]
fn abstract_proof_cannot_be_promoted_to_compiler_correctness() {
    let c = record();
    assert_eq!(c["implementation_bridge"]["status"], "open");
    assert_eq!(
        c["implementation_bridge"]["source"],
        "abstract_trace_and_result_model"
    );
    assert_eq!(
        c["implementation_bridge"]["target"],
        "affinescript_compiled_wasm_semantics"
    );
    assert_eq!(c["wasm_observation"]["status"], "historical_not_rechecked_here");
    assert_eq!(c["wasm_observation"]["wasm_byte_classes"], 2);
    assert_eq!(
        c["verification_scope"],
        "local_agda_kernel_check_only_not_a_ci_receipt"
    );
    let excluded = c["implementation_bridge"]["not_established"]
        .as_array()
        .expect("unproved obligations must be enumerated");
    assert!(excluded.contains(&json!("compiler_correctness")));
    assert!(excluded.contains(&json!("all_program_equivalence")));
}

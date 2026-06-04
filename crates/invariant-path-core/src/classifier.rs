// SPDX-License-Identifier: MPL-2.0
// Copyright (c) Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk>
#![forbid(unsafe_code)]

use crate::model::{ClaimCandidate, Classification, ClassificationOutcome, InvariantType};

const MODEL_TERMS: &[&str] = &["model", "simulation", "toy", "lab", "prototype"];
const REALITY_TERMS: &[&str] = &[
    "production",
    "real-world",
    "real world",
    "deployment",
    "all users",
];
const BENCHMARK_TERMS: &[&str] = &["benchmark", "leaderboard", "test set", "eval"];
const CAPABILITY_TERMS: &[&str] = &[
    "can reason",
    "general intelligence",
    "capability",
    "understands",
];
const THEOREM_TERMS: &[&str] = &["theorem", "proof", "lemma", "formal"];
const GUARANTEE_TERMS: &[&str] = &[
    "guarantee",
    "guarantees",
    "safe",
    "safety",
    "always",
    "certain",
];
const DATA_TERMS: &[&str] = &["data", "dataset", "study", "sample", "observation"];
const POLICY_TERMS: &[&str] = &["policy", "regulation", "mandate", "ban"];
const PROB_TERMS: &[&str] = &["probability", "likely", "uncertainty", "risk", "confidence"];
const CERTAINTY_TERMS: &[&str] = &["certain", "guarantee", "proves", "always", "definitely"];
const DESCRIPTIVE_TERMS: &[&str] = &["is", "are", "observed", "shows", "means"];
const NORMATIVE_TERMS: &[&str] = &["should", "must", "ought", "need to"];
const LOCAL_TERMS: &[&str] = &[
    "pilot",
    "single",
    "single test",
    "local",
    "subset",
    "case study",
    "model result",
];
const UNIVERSAL_TERMS: &[&str] = &["all", "every", "universal", "always", "everyone"];
const ASSUMPTION_TERMS: &[&str] = &["under assumptions", "assuming", "if we assume"];

pub fn classify_candidate(candidate: &ClaimCandidate) -> ClassificationOutcome {
    let source = candidate.source_text.to_lowercase();
    let target = candidate.target_text.to_lowercase();
    let combined = format!("{} {}", source, target);

    let invariant_type = infer_invariant_type(&source, &target, &candidate.trigger);
    let mut losses: Vec<String> = Vec::new();
    let mut classification = Classification::Abstain;
    let mut preserved = None;
    let mut break_condition = None;

    if has_any(&source, MODEL_TERMS) && has_any(&target, REALITY_TERMS) {
        losses.push("external_validity".to_string());
        classification = Classification::Overextended;
        preserved = Some(false);
        break_condition =
            Some("Model behavior does not directly transfer to deployment reality".to_string());
    }

    if has_any(&source, BENCHMARK_TERMS) && has_any(&target, CAPABILITY_TERMS) {
        push_loss(&mut losses, "task_transfer");
        classification = Classification::Overextended;
        preserved = Some(false);
        break_condition =
            Some("Benchmark performance is narrower than general capability claims".to_string());
    }

    if has_any(&source, THEOREM_TERMS) && has_any(&target, GUARANTEE_TERMS) {
        push_loss(&mut losses, "implementation_gap");
        classification = Classification::Overextended;
        preserved = Some(false);
        break_condition =
            Some("Formal theorem scope does not imply production guarantee".to_string());
    }

    if has_any(&source, DATA_TERMS)
        && (has_any(&target, POLICY_TERMS) || has_any(&target, NORMATIVE_TERMS))
    {
        push_loss(&mut losses, "normative_step");
        classification = Classification::Conflation;
        preserved = Some(false);
        break_condition = Some("Data alone does not encode policy or value tradeoffs".to_string());
    }

    if has_any(&source, PROB_TERMS) && has_any(&target, CERTAINTY_TERMS) {
        push_loss(&mut losses, "uncertainty");
        classification = Classification::Overextended;
        preserved = Some(false);
        break_condition = Some("Probabilistic evidence cannot justify certainty".to_string());
    }

    if has_any(&source, PROB_TERMS) && has_any(&target, NORMATIVE_TERMS) {
        push_loss(&mut losses, "uncertainty");
        push_loss(&mut losses, "value_judgment");
        classification = Classification::Conflation;
        preserved = Some(false);
        break_condition = Some(
            "Risk or uncertainty statements need explicit value framing before normative action"
                .to_string(),
        );
    }

    if has_any(&source, DESCRIPTIVE_TERMS) && has_any(&target, NORMATIVE_TERMS) {
        push_loss(&mut losses, "value_judgment");
        classification = Classification::Conflation;
        preserved = Some(false);
        break_condition = Some(
            "Descriptive claims require explicit normative bridge to justify prescriptions"
                .to_string(),
        );
    }

    if has_any(&source, LOCAL_TERMS) && has_any(&target, UNIVERSAL_TERMS) {
        push_loss(&mut losses, "scope");
        classification = Classification::Overextended;
        preserved = Some(false);
        break_condition = Some(
            "Local evidence cannot justify universal claim without additional support".to_string(),
        );
    }

    if (has_any(&combined, ASSUMPTION_TERMS) || candidate.trigger == "under assumptions")
        && has_any(&target, UNIVERSAL_TERMS)
    {
        push_loss(&mut losses, "assumptions");
        classification = Classification::Incomplete;
        preserved = Some(false);
        break_condition = Some(
            "Assumption-bound result was generalized without keeping assumptions explicit"
                .to_string(),
        );
    }

    if candidate.trigger == "clearly" || candidate.trigger == "obviously" {
        classification = Classification::Incomplete;
        preserved = None;
        push_loss(&mut losses, "missing_evidence");
        break_condition =
            Some("Rhetorical confidence marker without explicit supporting path".to_string());
    }

    if matches!(classification, Classification::Abstain)
        && (candidate.trigger == "therefore"
            || candidate.trigger == "implies"
            || candidate.trigger == "this shows")
    {
        classification = Classification::ValidPath;
        preserved = Some(true);
    }

    ClassificationOutcome {
        invariant_type,
        preserved,
        losses,
        break_condition,
        classification,
    }
}

pub fn infer_invariant_type(source: &str, target: &str, trigger: &str) -> InvariantType {
    let text = format!("{} {} {}", source, target, trigger);

    if has_any(&text, &["should", "must", "policy", "ought"]) {
        return InvariantType::NormativeBridge;
    }
    if has_any(&text, &["because", "causes", "leads to", "therefore"]) {
        return InvariantType::CausalRelationship;
    }
    if has_any(
        &text,
        &["probability", "likely", "confidence", "distribution"],
    ) {
        return InvariantType::StatisticalRelationship;
    }
    if has_any(&text, &["implies", "proof", "theorem", "logical"]) {
        return InvariantType::LogicalImplication;
    }
    if has_any(&text, &["mechanism", "in practice", "works by", "pathway"]) {
        return InvariantType::Mechanism;
    }
    if has_any(&text, &["risk", "uncertainty", "safety", "hazard"]) {
        return InvariantType::RiskProbability;
    }
    if has_any(&text, &["cost", "budget", "resource", "easy", "transition"]) {
        return InvariantType::ResourceBudget;
    }

    InvariantType::Provenance
}

fn has_any(text: &str, words: &[&str]) -> bool {
    words.iter().any(|word| text.contains(word))
}

fn push_loss(losses: &mut Vec<String>, loss: &str) {
    if !losses.iter().any(|entry| entry == loss) {
        losses.push(loss.to_string());
    }
}

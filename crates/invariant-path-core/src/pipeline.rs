// SPDX-License-Identifier: MPL-2.0
#![forbid(unsafe_code)]

use crate::classifier::classify_candidate;
use crate::extractor::extract_candidates;
use crate::model::{make_annotation_id, now_rfc3339, Annotation, Status, Visibility};

/// Scan several artifacts in one pass, e.g. every root document of a corpus.
/// Each root is an `(artifact_uri, text)` pair; results are concatenated in
/// input order so annotation IDs remain stable per artifact.
pub fn scan_multi_root(
    roots: &[(&str, &str)],
    created_by: &str,
    visibility: Visibility,
) -> Vec<Annotation> {
    roots
        .iter()
        .flat_map(|(artifact_uri, text)| {
            scan_artifact(artifact_uri, text, created_by, visibility.clone())
        })
        .collect()
}

pub fn scan_artifact(
    artifact_uri: &str,
    text: &str,
    created_by: &str,
    visibility: Visibility,
) -> Vec<Annotation> {
    extract_candidates(text)
        .into_iter()
        .map(|candidate| {
            let outcome = classify_candidate(&candidate);
            Annotation {
                id: make_annotation_id(
                    artifact_uri,
                    &candidate.span,
                    &candidate.source_text,
                    &candidate.target_text,
                ),
                artifact_uri: artifact_uri.to_string(),
                span: candidate.span,
                source_text: candidate.source_text,
                target_text: candidate.target_text,
                path_description: candidate.path_description,
                invariant_type: outcome.invariant_type,
                preserved: outcome.preserved,
                losses: outcome.losses,
                break_condition: outcome.break_condition,
                classification: outcome.classification,
                visibility: visibility.clone(),
                status: Status::Open,
                created_by: created_by.to_string(),
                updated_at: now_rfc3339(),
                notes: "auto-generated heuristic suggestion; editable".to_string(),
            }
        })
        .collect()
}

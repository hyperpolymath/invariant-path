// SPDX-License-Identifier: MPL-2.0
// Copyright (c) Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk>
#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Classification {
    ValidPath,
    Overextended,
    Conflation,
    Incomplete,
    Abstain,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum InvariantType {
    CausalRelationship,
    StatisticalRelationship,
    LogicalImplication,
    Mechanism,
    RiskProbability,
    NormativeBridge,
    ResourceBudget,
    Provenance,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Visibility {
    Private,
    SharedTeam,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    Open,
    Accepted,
    Dismissed,
    Clarified,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ClaimCandidate {
    pub span: Span,
    pub source_text: String,
    pub target_text: String,
    pub trigger: String,
    pub path_description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ClassificationOutcome {
    pub invariant_type: InvariantType,
    pub preserved: Option<bool>,
    pub losses: Vec<String>,
    pub break_condition: Option<String>,
    pub classification: Classification,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Annotation {
    pub id: String,
    pub artifact_uri: String,
    pub span: Span,
    pub source_text: String,
    pub target_text: String,
    pub path_description: String,
    pub invariant_type: InvariantType,
    pub preserved: Option<bool>,
    pub losses: Vec<String>,
    pub break_condition: Option<String>,
    pub classification: Classification,
    pub visibility: Visibility,
    pub status: Status,
    pub created_by: String,
    pub updated_at: String,
    pub notes: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OverlayState {
    pub enabled: bool,
    pub updated_at: String,
}

pub fn now_rfc3339() -> String {
    OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .unwrap_or_else(|_| "1970-01-01T00:00:00Z".to_string())
}

pub fn make_annotation_id(artifact_uri: &str, span: &Span, source: &str, target: &str) -> String {
    let mut hasher = DefaultHasher::new();
    artifact_uri.hash(&mut hasher);
    span.start.hash(&mut hasher);
    span.end.hash(&mut hasher);
    source.hash(&mut hasher);
    target.hash(&mut hasher);
    format!("ip-{:016x}", hasher.finish())
}

// SPDX-License-Identifier: MPL-2.0
#![forbid(unsafe_code)]

//! Document-claim grounding.
//!
//! invariant-path's existing classifier looks at *rhetorical* invariants
//! (overextension, conflation, etc.) inside prose. This module is the
//! complementary lens: it extracts *factual* claims that documentation
//! makes about the world — usually about file paths, repository state,
//! or external command results — and grounds them against ground truth.
//!
//! The use case is "panic-attack for prose": run it over a corpus
//! (`standards/`, the PMPL text, anything you have written down), get
//! back a JSONL report of every claim and whether it can be grounded.
//! Wire it into CI just like `panic-attack assail`.

use crate::model::{now_rfc3339, Span};
use once_cell::sync::Lazy;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};

/// A factual claim extracted from a documentation file.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DocClaim {
    /// Stable id derived from the source URI + span + claim text.
    pub id: String,
    /// `repo://<relative path>` or `file://<abs>` form.
    pub source_uri: String,
    /// 1-based line number where the claim was matched.
    pub line: usize,
    pub span: Span,
    /// The raw claim text as it appeared in the source (single line).
    pub raw: String,
    /// Structured predicate the grounder will resolve.
    pub predicate: Predicate,
}

/// What ground truth the claim is asserting.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Predicate {
    /// The named path should exist (file or directory).
    FileExists { path: String },
    /// The named path should exist AND its contents should match `pattern`.
    PathContainsRegex { path: String, pattern: String },
    /// An A2ML key (dotted path) should be present in an A2ML file.
    A2mlKeyPresent { file: String, key: String },
    /// A claim about command output we can only verify by execution.
    /// Marked unknown by default; `--allow-exec` may resolve it.
    CommandCleanClaim { command: String },
    /// A claim we recognised as a claim but cannot decompose into a
    /// machine-checkable predicate. Reported but not failed.
    Unknown { reason: String },
}

/// Result of grounding a claim.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "result", rename_all = "snake_case")]
pub enum ClaimResult {
    /// Predicate evaluated and held.
    Grounded,
    /// Predicate evaluated and failed.
    Ungrounded { reason: String },
    /// Predicate could not be evaluated (no resolver, exec disabled, etc.).
    Unknown { reason: String },
}

/// One row in the JSONL report file.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DocClaimRecord {
    pub claim: DocClaim,
    pub result: ClaimResult,
    pub assessed_at: String,
}

/// Knobs for the grounder. Defaults are CI-safe (no command execution).
#[derive(Debug, Clone)]
pub struct GrounderConfig {
    /// If true, `CommandCleanClaim` predicates may invoke the named
    /// command. Off by default — see CLI `--allow-exec`.
    pub allow_exec: bool,
    /// Max bytes to read from a file when checking `PathContainsRegex`.
    pub max_read_bytes: usize,
}

impl Default for GrounderConfig {
    fn default() -> Self {
        Self {
            allow_exec: false,
            max_read_bytes: 4 * 1024 * 1024,
        }
    }
}

// ---- recognisers ----

/// A backtick-quoted path with a recognised extension.
/// Matches the very common pattern of estate docs naming a file by path.
static BACKTICK_PATH: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r"`([A-Za-z0-9_.\-/]+\.(?:adoc|md|toml|rs|hs|ml|jl|sh|a2ml|ncl|scm|nix|cabal|just|yml|yaml|idr|lean|v|zig|gleam|ex|ml|cs|py))`",
    )
    .expect("BACKTICK_PATH regex")
});

/// "panic-attack assail" / "Hypatia zero findings" / similar canned
/// command-output claims. The grounder treats these as Unknown unless
/// `--allow-exec` is set.
static COMMAND_CLEAN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)\b(panic-attack assail|hypatia(?: scan)?|just (?:test|build|ci))\b\s+(?:returns? )?(?:clean|passes?|green|zero findings?)")
        .expect("COMMAND_CLEAN regex")
});

/// "STATE.a2ml" / "0-AI-MANIFEST.a2ml" mentioned with a presence verb.
static A2ML_PRESENT: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"`?([A-Za-z0-9_.\-/]+\.a2ml)`?\s+(?:present|exists|wired|validated)")
        .expect("A2ML_PRESENT regex")
});

/// Extract claims from a single file's text.
///
/// Each `\n` boundary increments the line counter. Spans are byte
/// offsets into `text`.
pub fn extract_claims(text: &str, source_uri: &str) -> Vec<DocClaim> {
    let mut out: Vec<DocClaim> = Vec::new();
    let mut line: usize = 1;
    let mut line_start: usize = 0;
    let bytes = text.as_bytes();

    let recognise_at = |out: &mut Vec<DocClaim>, line: usize, slice: &str, slice_start: usize| {
        // backtick paths
        for cap in BACKTICK_PATH.captures_iter(slice) {
            let m = cap.get(0).unwrap();
            let path = cap.get(1).unwrap().as_str().to_string();
            let raw = m.as_str().to_string();
            let span = Span {
                start: slice_start + m.start(),
                end: slice_start + m.end(),
            };
            let predicate = Predicate::FileExists { path: path.clone() };
            out.push(make_claim(source_uri, line, span, raw, predicate));
        }
        // a2ml presence
        for cap in A2ML_PRESENT.captures_iter(slice) {
            let m = cap.get(0).unwrap();
            let path = cap.get(1).unwrap().as_str().to_string();
            let raw = m.as_str().to_string();
            let span = Span {
                start: slice_start + m.start(),
                end: slice_start + m.end(),
            };
            let predicate = Predicate::FileExists { path };
            out.push(make_claim(source_uri, line, span, raw, predicate));
        }
        // command-clean
        for cap in COMMAND_CLEAN.captures_iter(slice) {
            let m = cap.get(0).unwrap();
            let command = cap.get(1).unwrap().as_str().to_string();
            let raw = m.as_str().to_string();
            let span = Span {
                start: slice_start + m.start(),
                end: slice_start + m.end(),
            };
            let predicate = Predicate::CommandCleanClaim { command };
            out.push(make_claim(source_uri, line, span, raw, predicate));
        }
    };

    for (i, &b) in bytes.iter().enumerate() {
        if b == b'\n' {
            let slice = &text[line_start..i];
            recognise_at(&mut out, line, slice, line_start);
            line += 1;
            line_start = i + 1;
        }
    }
    if line_start < bytes.len() {
        let slice = &text[line_start..];
        recognise_at(&mut out, line, slice, line_start);
    }
    dedupe_in_place(&mut out);
    out
}

fn dedupe_in_place(claims: &mut Vec<DocClaim>) {
    let mut seen = std::collections::HashSet::new();
    claims.retain(|c| seen.insert(c.id.clone()));
}

fn make_claim(
    source_uri: &str,
    line: usize,
    span: Span,
    raw: String,
    predicate: Predicate,
) -> DocClaim {
    let id = make_claim_id(source_uri, &span, &raw);
    DocClaim {
        id,
        source_uri: source_uri.to_string(),
        line,
        span,
        raw,
        predicate,
    }
}

fn make_claim_id(source_uri: &str, span: &Span, raw: &str) -> String {
    let mut h = DefaultHasher::new();
    source_uri.hash(&mut h);
    span.start.hash(&mut h);
    span.end.hash(&mut h);
    raw.hash(&mut h);
    format!("ipdc-{:016x}", h.finish())
}

/// Resolve one claim against the file system rooted at any of the `roots`.
///
/// Multi-root grounding: the predicate (e.g. FileExists) is checked against
/// each root in order. The first match is used. This allows documentation
/// in `standards/` to reference files in `hypatia/` if both are roots.
pub fn ground_claim(claim: &DocClaim, roots: &[PathBuf], cfg: &GrounderConfig) -> ClaimResult {
    match &claim.predicate {
        Predicate::FileExists { path } => {
            let candidate = resolve_across_roots(roots, path);
            if candidate.map(|c| c.exists()).unwrap_or(false) {
                ClaimResult::Grounded
            } else {
                ClaimResult::Ungrounded {
                    reason: format!("path not found under any corpus root: {}", path),
                }
            }
        }
        Predicate::PathContainsRegex { path, pattern } => {
            let candidate = match resolve_across_roots(roots, path) {
                Some(c) if c.exists() => c,
                _ => {
                    return ClaimResult::Ungrounded {
                        reason: format!("path not found: {}", path),
                    };
                }
            };
            let body = match read_capped(&candidate, cfg.max_read_bytes) {
                Ok(s) => s,
                Err(e) => {
                    return ClaimResult::Unknown {
                        reason: format!("read error: {}", e),
                    }
                }
            };
            match Regex::new(pattern) {
                Ok(re) if re.is_match(&body) => ClaimResult::Grounded,
                Ok(_) => ClaimResult::Ungrounded {
                    reason: format!("pattern not found in {}: {}", path, pattern),
                },
                Err(e) => ClaimResult::Unknown {
                    reason: format!("invalid pattern: {}", e),
                },
            }
        }
        Predicate::A2mlKeyPresent { file, key } => {
            let candidate = match resolve_across_roots(roots, file) {
                Some(c) if c.exists() => c,
                _ => {
                    return ClaimResult::Ungrounded {
                        reason: format!("a2ml file not found: {}", file),
                    };
                }
            };
            let body = match read_capped(&candidate, cfg.max_read_bytes) {
                Ok(s) => s,
                Err(e) => {
                    return ClaimResult::Unknown {
                        reason: format!("a2ml read error: {}", e),
                    }
                }
            };
            // v1: lexical key check (a real A2ML parser comes from the
            // standards repo's a2ml-rs crate later).
            let needle = format!("({}", key.split('.').next().unwrap_or(key));
            if body.contains(&needle) {
                ClaimResult::Grounded
            } else {
                ClaimResult::Ungrounded {
                    reason: format!("a2ml key not found: {} in {}", key, file),
                }
            }
        }
        Predicate::CommandCleanClaim { command } => {
            if !cfg.allow_exec {
                return ClaimResult::Unknown {
                    reason: format!(
                        "command claim not executed (use --allow-exec to verify): {}",
                        command
                    ),
                };
            }
            ClaimResult::Unknown {
                reason: format!(
                    "exec resolver not yet implemented for: {} (planned)",
                    command
                ),
            }
        }
        Predicate::Unknown { reason } => ClaimResult::Unknown {
            reason: reason.clone(),
        },
    }
}

fn resolve_across_roots(roots: &[PathBuf], path: &str) -> Option<PathBuf> {
    if Path::new(path).is_absolute() {
        return Some(PathBuf::from(path));
    }
    for root in roots {
        let candidate = root.join(path);
        if candidate.exists() {
            return Some(candidate);
        }
    }
    // Return the first root as a fallback even if it doesn't exist,
    // to preserve current diagnostic behavior in ground_claim.
    roots.first().map(|r| r.join(path))
}

fn read_capped(path: &Path, cap: usize) -> std::io::Result<String> {
    let bytes = fs::read(path)?;
    let cut = bytes.len().min(cap);
    Ok(String::from_utf8_lossy(&bytes[..cut]).into_owned())
}

/// Walk a directory and return claim+result pairs for every recognised
/// file. v1 supports `.md` and `.adoc`; the matrix can grow.
///
/// `scan_root` is the directory being scanned for claims.
/// `ground_roots` are the roots used to resolve those claims.
pub fn scan_corpus(
    scan_root: &Path,
    ground_roots: &[PathBuf],
    cfg: &GrounderConfig,
) -> Vec<DocClaimRecord> {
    let mut out: Vec<DocClaimRecord> = Vec::new();
    walk(scan_root, scan_root, ground_roots, &mut out, cfg);
    out
}

fn walk(
    scan_root: &Path,
    dir: &Path,
    ground_roots: &[PathBuf],
    out: &mut Vec<DocClaimRecord>,
    cfg: &GrounderConfig,
) {
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            if name.starts_with('.') || name == "target" || name == "node_modules" {
                continue;
            }
        }
        if path.is_dir() {
            walk(scan_root, &path, ground_roots, out, cfg);
            continue;
        }
        let ext = match path.extension().and_then(|e| e.to_str()) {
            Some(e) => e.to_lowercase(),
            None => continue,
        };
        if ext != "md" && ext != "adoc" {
            continue;
        }
        let text = match fs::read_to_string(&path) {
            Ok(t) => t,
            Err(_) => continue,
        };
        let rel = path.strip_prefix(scan_root).unwrap_or(&path).to_string_lossy();
        let source_uri = format!("repo://{}", rel);
        let claims = extract_claims(&text, &source_uri);
        for claim in claims {
            let result = ground_claim(&claim, ground_roots, cfg);
            out.push(DocClaimRecord {
                claim,
                result,
                assessed_at: now_rfc3339(),
            });
        }
    }
}

/// Convenience: split records into ungrounded vs other for CI exit
/// codes. The "panic-attack for docs" semantics: any Ungrounded claim
/// is a CI failure; Unknown is informational.
pub fn partition_failures(records: &[DocClaimRecord]) -> (Vec<&DocClaimRecord>, Vec<&DocClaimRecord>) {
    let mut failures = Vec::new();
    let mut ok_or_unknown = Vec::new();
    for r in records {
        match &r.result {
            ClaimResult::Ungrounded { .. } => failures.push(r),
            _ => ok_or_unknown.push(r),
        }
    }
    (failures, ok_or_unknown)
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn proptest_extract_claims_no_panic(s in "\\PC*") {
            let _ = extract_claims(&s, "repo://proptest.md");
        }
    }

    #[test]
    fn extracts_backtick_path() {
        let text = "The proof suite lives at `templates/CANONICAL-PROOF-SUITE.adoc`.";
        let claims = extract_claims(text, "repo://test.adoc");
        assert_eq!(claims.len(), 1);
        match &claims[0].predicate {
            Predicate::FileExists { path } => {
                assert_eq!(path, "templates/CANONICAL-PROOF-SUITE.adoc")
            }
            other => panic!("expected FileExists, got {:?}", other),
        }
    }

    #[test]
    fn extracts_a2ml_present() {
        let text = "The `0-AI-MANIFEST.a2ml` is present and validated.";
        let claims = extract_claims(text, "repo://x.adoc");
        assert!(claims.iter().any(|c| matches!(
            &c.predicate,
            Predicate::FileExists { path } if path == "0-AI-MANIFEST.a2ml"
        )));
    }

    #[test]
    fn extracts_command_clean_claim() {
        let text = "panic-attack assail returns clean across all severities.";
        let claims = extract_claims(text, "repo://y.adoc");
        assert!(claims
            .iter()
            .any(|c| matches!(c.predicate, Predicate::CommandCleanClaim { .. })));
    }

    #[test]
    fn ground_existing_path() {
        let temp = std::env::temp_dir().join(format!("ipdc-test-{}", std::process::id()));
        std::fs::create_dir_all(temp.join("templates")).unwrap();
        std::fs::write(temp.join("templates/X.adoc"), "hi").unwrap();
        let claim = DocClaim {
            id: "t".into(),
            source_uri: "repo://t.adoc".into(),
            line: 1,
            span: Span { start: 0, end: 0 },
            raw: "`templates/X.adoc`".into(),
            predicate: Predicate::FileExists {
                path: "templates/X.adoc".into(),
            },
        };
        let cfg = GrounderConfig::default();
        let roots = vec![temp.clone()];
        assert_eq!(ground_claim(&claim, &roots, &cfg), ClaimResult::Grounded);
        std::fs::remove_dir_all(&temp).unwrap();
    }

    #[test]
    fn ground_missing_path_is_ungrounded() {
        let temp = std::env::temp_dir().join(format!("ipdc-test2-{}", std::process::id()));
        std::fs::create_dir_all(&temp).unwrap();
        let claim = DocClaim {
            id: "t".into(),
            source_uri: "repo://t.adoc".into(),
            line: 1,
            span: Span { start: 0, end: 0 },
            raw: "`templates/MISSING.adoc`".into(),
            predicate: Predicate::FileExists {
                path: "templates/MISSING.adoc".into(),
            },
        };
        let cfg = GrounderConfig::default();
        let roots = vec![temp.clone()];
        match ground_claim(&claim, &roots, &cfg) {
            ClaimResult::Ungrounded { .. } => {}
            other => panic!("expected Ungrounded, got {:?}", other),
        }
        std::fs::remove_dir_all(&temp).unwrap();
    }

    #[test]
    fn ground_multi_root_resolution() {
        let temp1 = std::env::temp_dir().join(format!("ipdc-multi1-{}", std::process::id()));
        let temp2 = std::env::temp_dir().join(format!("ipdc-multi2-{}", std::process::id()));
        std::fs::create_dir_all(&temp1).unwrap();
        std::fs::create_dir_all(&temp2).unwrap();
        std::fs::write(temp2.join("Y.adoc"), "hi").unwrap();
        
        let claim = DocClaim {
            id: "t".into(),
            source_uri: "repo://t.adoc".into(),
            line: 1,
            span: Span { start: 0, end: 0 },
            raw: "`Y.adoc`".into(),
            predicate: Predicate::FileExists {
                path: "Y.adoc".into(),
            },
        };
        let cfg = GrounderConfig::default();
        let roots = vec![temp1.clone(), temp2.clone()];
        assert_eq!(ground_claim(&claim, &roots, &cfg), ClaimResult::Grounded);
        
        std::fs::remove_dir_all(&temp1).unwrap();
        std::fs::remove_dir_all(&temp2).unwrap();
    }

    #[test]
    fn command_claim_is_unknown_without_exec() {
        let claim = DocClaim {
            id: "t".into(),
            source_uri: "repo://t.adoc".into(),
            line: 1,
            span: Span { start: 0, end: 0 },
            raw: "panic-attack assail clean".into(),
            predicate: Predicate::CommandCleanClaim {
                command: "panic-attack assail".into(),
            },
        };
        let cfg = GrounderConfig::default();
        let roots = vec![PathBuf::from("/")];
        match ground_claim(&claim, &roots, &cfg) {
            ClaimResult::Unknown { .. } => {}
            other => panic!("expected Unknown, got {:?}", other),
        }
    }
}

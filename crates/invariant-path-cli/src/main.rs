// SPDX-License-Identifier: PMPL-1.0-or-later
#![forbid(unsafe_code)]

use clap::{Parser, Subcommand, ValueEnum};
use invariant_path_core::annotations::AnnotationStore;
use invariant_path_core::doc_claims::{
    partition_failures, scan_corpus, ClaimResult, GrounderConfig,
};
use invariant_path_core::model::{
    make_annotation_id, now_rfc3339, Annotation, Classification, InvariantType, Span, Status,
    Visibility,
};
use invariant_path_core::pipeline::scan_artifact;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(name = "invariant-path")]
#[command(about = "Claim-path debugger overlay for repository artifacts")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Extract and classify candidate claim transitions.
    Scan(ScanArgs),
    /// Manage stored annotations.
    Annotations {
        #[command(subcommand)]
        command: AnnotationCommand,
    },
    /// Toggle overlay visibility state.
    Overlay {
        #[command(subcommand)]
        command: OverlayCommand,
    },
    /// Show built-in repo profiles.
    Profiles,
    /// Ground factual claims in a documentation corpus against the file
    /// system. The "panic-attack for prose" mode: scans .md/.adoc under
    /// `--root`, extracts file-existence and command-clean claims, and
    /// fails on any ungrounded claim. Use this on standards/, on the
    /// PMPL text, or on any prose corpus you need to keep honest.
    DocClaims(DocClaimsArgs),
}

#[derive(Debug, clap::Args)]
struct DocClaimsArgs {
    /// Roots to scan recursively for documentation files.
    /// If omitted, defaults to the first grounding root.
    #[arg(long = "scan-root")]
    scan_roots: Vec<PathBuf>,

    /// Grounding roots used to resolve file and path claims.
    /// At least one root is required.
    #[arg(long = "root", required = true)]
    roots: Vec<PathBuf>,

    /// Optional output JSONL path. If omitted, prints to stdout.
    #[arg(long)]
    out: Option<PathBuf>,

    /// Permit the grounder to execute named commands when verifying
    /// `CommandCleanClaim` predicates. Off by default — CI-safe.
    #[arg(long, default_value_t = false)]
    allow_exec: bool,

    /// Emit a compact JSON summary on stdout (in addition to JSONL).
    #[arg(long, default_value_t = false)]
    json: bool,

    /// Treat unknown (unresolvable) claims as failures too. Off by
    /// default — unknown is informational, ungrounded is the hard fail.
    #[arg(long, default_value_t = false)]
    strict_unknown: bool,
}

#[derive(Debug, clap::Args)]
struct ScanArgs {
    /// Path to artifact file.
    #[arg(long)]
    file: Option<PathBuf>,

    /// Raw artifact text.
    #[arg(long)]
    text: Option<String>,

    /// Artifact URI (defaults to repo://<file>).
    #[arg(long)]
    artifact_uri: Option<String>,

    #[arg(long, value_enum, default_value = "generic")]
    profile: ProfilePreset,

    #[arg(long, value_enum)]
    visibility: Option<VisibilityArg>,

    #[arg(long, default_value = "local-user")]
    created_by: String,

    #[arg(long, default_value = ".invariant-path")]
    store: PathBuf,

    /// Persist generated suggestions.
    #[arg(long)]
    write: bool,

    /// Emit JSON output.
    #[arg(long)]
    json: bool,

    /// Max suggestions to emit.
    #[arg(long)]
    max: Option<usize>,
}

#[derive(Debug, Subcommand)]
enum AnnotationCommand {
    /// List annotations.
    List(ListArgs),
    /// Add a manual annotation.
    Add(AddArgs),
    /// Accept a suggestion.
    Accept(IdArgs),
    /// Dismiss for self (forces visibility private).
    Dismiss(IdArgs),
    /// Mark as clarified.
    Clarify(IdArgs),
    /// Update annotation fields.
    Update(UpdateArgs),
}

#[derive(Debug, clap::Args)]
struct ListArgs {
    #[arg(long, default_value = ".invariant-path")]
    store: PathBuf,

    #[arg(long)]
    artifact_uri: Option<String>,

    #[arg(long, value_enum)]
    visibility: Option<VisibilityArg>,

    #[arg(long, value_enum)]
    status: Option<StatusArg>,

    #[arg(long)]
    json: bool,
}

#[derive(Debug, clap::Args)]
struct AddArgs {
    #[arg(long, default_value = ".invariant-path")]
    store: PathBuf,

    #[arg(long)]
    artifact_uri: String,

    #[arg(long)]
    source_text: String,

    #[arg(long)]
    target_text: String,

    #[arg(long)]
    path_description: Option<String>,

    #[arg(long, value_enum)]
    invariant_type: InvariantTypeArg,

    #[arg(long, value_enum)]
    classification: ClassificationArg,

    #[arg(long, value_enum, default_value = "private")]
    visibility: VisibilityArg,

    #[arg(long, value_enum, default_value = "open")]
    status: StatusArg,

    #[arg(long)]
    preserved: Option<bool>,

    #[arg(long = "loss")]
    losses: Vec<String>,

    #[arg(long)]
    break_condition: Option<String>,

    #[arg(long, default_value = "local-user")]
    created_by: String,

    #[arg(long, default_value = "")]
    notes: String,

    #[arg(long, default_value_t = 0)]
    span_start: usize,

    #[arg(long, default_value_t = 0)]
    span_end: usize,

    #[arg(long)]
    json: bool,
}

#[derive(Debug, clap::Args)]
struct IdArgs {
    #[arg(long, default_value = ".invariant-path")]
    store: PathBuf,

    #[arg(long)]
    id: String,

    #[arg(long)]
    notes: Option<String>,

    #[arg(long)]
    json: bool,
}

#[derive(Debug, clap::Args)]
struct UpdateArgs {
    #[arg(long, default_value = ".invariant-path")]
    store: PathBuf,

    #[arg(long)]
    id: String,

    #[arg(long, value_enum)]
    invariant_type: Option<InvariantTypeArg>,

    #[arg(long, value_enum)]
    classification: Option<ClassificationArg>,

    #[arg(long, value_enum)]
    visibility: Option<VisibilityArg>,

    #[arg(long, value_enum)]
    status: Option<StatusArg>,

    #[arg(long)]
    preserved: Option<String>,

    #[arg(long = "add-loss")]
    add_losses: Vec<String>,

    #[arg(long)]
    break_condition: Option<String>,

    #[arg(long)]
    notes: Option<String>,

    #[arg(long)]
    json: bool,
}

#[derive(Debug, Subcommand)]
enum OverlayCommand {
    Status(OverlayStatusArgs),
    Toggle(OverlayToggleArgs),
}

#[derive(Debug, clap::Args)]
struct OverlayStatusArgs {
    #[arg(long, default_value = ".invariant-path")]
    store: PathBuf,

    #[arg(long)]
    json: bool,
}

#[derive(Debug, clap::Args)]
struct OverlayToggleArgs {
    #[arg(long, default_value = ".invariant-path")]
    store: PathBuf,

    #[arg(long, value_enum)]
    state: ToggleArg,

    #[arg(long)]
    json: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
enum ProfilePreset {
    Generic,
    Echidna,
    Panll,
    Hypatia,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum VisibilityArg {
    Private,
    #[value(name = "shared_team")]
    SharedTeam,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum StatusArg {
    Open,
    Accepted,
    Dismissed,
    Clarified,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum InvariantTypeArg {
    CausalRelationship,
    StatisticalRelationship,
    LogicalImplication,
    Mechanism,
    RiskProbability,
    NormativeBridge,
    ResourceBudget,
    Provenance,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum ClassificationArg {
    ValidPath,
    Overextended,
    Conflation,
    Incomplete,
    Abstain,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum ToggleArg {
    On,
    Off,
}

#[derive(Debug)]
struct ProfileConfig {
    name: &'static str,
    default_visibility: Visibility,
    include_suffixes: &'static [&'static str],
    focus: &'static [&'static str],
}

fn main() -> Result<(), String> {
    let cli = Cli::parse();

    match cli.command {
        Command::Scan(args) => run_scan(args),
        Command::Annotations { command } => run_annotations(command),
        Command::Overlay { command } => run_overlay(command),
        Command::Profiles => {
            let profiles = [
                profile_config(ProfilePreset::Generic),
                profile_config(ProfilePreset::Echidna),
                profile_config(ProfilePreset::Panll),
                profile_config(ProfilePreset::Hypatia),
            ];
            for profile in profiles {
                println!(
                    "{}: default_visibility={:?}, include={:?}, focus={:?}",
                    profile.name,
                    profile.default_visibility,
                    profile.include_suffixes,
                    profile.focus
                );
            }
            Ok(())
        }
        Command::DocClaims(args) => run_doc_claims(args),
    }
}

fn run_doc_claims(args: DocClaimsArgs) -> Result<(), String> {
    for root in &args.roots {
        if !root.exists() {
            return Err(format!("grounding root not found: {}", root.display()));
        }
    }
    let scan_roots = if args.scan_roots.is_empty() {
        vec![args.roots[0].clone()]
    } else {
        args.scan_roots.clone()
    };
    for s_root in &scan_roots {
        if !s_root.exists() {
            return Err(format!("scan root not found: {}", s_root.display()));
        }
    }

    let cfg = GrounderConfig {
        allow_exec: args.allow_exec,
        ..GrounderConfig::default()
    };

    let mut records = Vec::new();
    for s_root in &scan_roots {
        records.extend(scan_corpus(s_root, &args.roots, &cfg));
    }

    if let Some(out_path) = &args.out {
        if let Some(parent) = out_path.parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent).map_err(|e| format!("create out parent: {e}"))?;
            }
        }
        let mut buf = String::new();
        for r in &records {
            buf.push_str(&serde_json::to_string(r).map_err(|e| format!("encode: {e}"))?);
            buf.push('\n');
        }
        fs::write(out_path, buf).map_err(|e| format!("write {}: {e}", out_path.display()))?;
    } else if !args.json {
        for r in &records {
            let result_tag = match &r.result {
                ClaimResult::Grounded => "OK     ",
                ClaimResult::Ungrounded { .. } => "FAIL   ",
                ClaimResult::Unknown { .. } => "UNKNOWN",
            };
            println!(
                "{} {}:{}  {}",
                result_tag, r.claim.source_uri, r.claim.line, r.claim.raw
            );
            if let ClaimResult::Ungrounded { reason } | ClaimResult::Unknown { reason } =
                &r.result
            {
                println!("         reason: {reason}");
            }
        }
    }

    let total = records.len();
    let (failures, _) = partition_failures(&records);
    let unknowns = records
        .iter()
        .filter(|r| matches!(r.result, ClaimResult::Unknown { .. }))
        .count();
    let grounded = records
        .iter()
        .filter(|r| matches!(r.result, ClaimResult::Grounded))
        .count();

    if args.json {
        let summary = serde_json::json!({
            "total": total,
            "grounded": grounded,
            "ungrounded": failures.len(),
            "unknown": unknowns,
            "out": args.out.as_ref().map(|p| p.display().to_string()),
        });
        println!("{}", summary);
    } else {
        eprintln!(
            "doc-claims: {grounded} grounded / {} ungrounded / {unknowns} unknown / {total} total",
            failures.len()
        );
    }

    let hard_fail = failures.len() + if args.strict_unknown { unknowns } else { 0 };
    if hard_fail > 0 {
        std::process::exit(2);
    }
    Ok(())
}

fn run_scan(args: ScanArgs) -> Result<(), String> {
    let profile = profile_config(args.profile);
    let text = if let Some(input) = args.text {
        input
    } else if let Some(path) = &args.file {
        fs::read_to_string(path)
            .map_err(|err| format!("failed reading file {}: {}", path.display(), err))?
    } else {
        return Err("either --file or --text is required".to_string());
    };

    let artifact_uri = if let Some(uri) = args.artifact_uri {
        uri
    } else if let Some(path) = &args.file {
        format!("repo://{}", path.display())
    } else {
        "repo://inline-input".to_string()
    };

    if !path_matches_profile(&artifact_uri, profile.include_suffixes)
        && args.profile != ProfilePreset::Generic
    {
        eprintln!(
            "warning: artifact_uri '{}' does not match preferred profile suffixes {:?}",
            artifact_uri, profile.include_suffixes
        );
    }

    let visibility = args
        .visibility
        .map(Visibility::from)
        .unwrap_or_else(|| profile.default_visibility.clone());

    let mut suggestions = scan_artifact(&artifact_uri, &text, &args.created_by, visibility);
    if let Some(max) = args.max {
        suggestions.truncate(max);
    }

    if args.write {
        let store = AnnotationStore::new(&args.store)
            .map_err(|err| format!("failed opening store {}: {}", args.store.display(), err))?;
        store
            .save_many(&suggestions)
            .map_err(|err| format!("failed saving suggestions: {}", err))?;
    }

    if args.json {
        println!(
            "{}",
            serde_json::to_string_pretty(&suggestions)
                .map_err(|err| format!("failed to serialize scan output: {}", err))?
        );
    } else {
        println!(
            "profile={} suggestions={} artifact={}",
            profile.name,
            suggestions.len(),
            artifact_uri
        );
        for ann in &suggestions {
            println!(
                "- [{}] {} {}",
                ann.id,
                enum_label_classification(&ann.classification),
                enum_label_invariant(&ann.invariant_type)
            );
            println!("  source: {}", ann.source_text);
            println!("  target: {}", ann.target_text);
            println!("  losses: {:?}", ann.losses);
            if let Some(break_condition) = &ann.break_condition {
                println!("  break: {}", break_condition);
            }
        }
    }

    Ok(())
}

fn run_annotations(command: AnnotationCommand) -> Result<(), String> {
    match command {
        AnnotationCommand::List(args) => {
            let store = AnnotationStore::new(&args.store)
                .map_err(|err| format!("failed opening store {}: {}", args.store.display(), err))?;
            let mut items = store
                .list()
                .map_err(|err| format!("failed listing annotations: {}", err))?;

            if let Some(artifact_uri) = args.artifact_uri {
                items.retain(|item| item.artifact_uri == artifact_uri);
            }
            if let Some(visibility) = args.visibility {
                let wanted = Visibility::from(visibility);
                items.retain(|item| item.visibility == wanted);
            }
            if let Some(status) = args.status {
                let wanted = Status::from(status);
                items.retain(|item| item.status == wanted);
            }

            if args.json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&items)
                        .map_err(|err| format!("failed to serialize annotations: {}", err))?
                );
            } else {
                println!("annotations={}", items.len());
                for ann in items {
                    println!(
                        "- {} [{} {:?}] {}",
                        ann.id,
                        enum_label_status(&ann.status),
                        ann.visibility,
                        ann.path_description
                    );
                }
            }
            Ok(())
        }
        AnnotationCommand::Add(args) => {
            let store = AnnotationStore::new(&args.store)
                .map_err(|err| format!("failed opening store {}: {}", args.store.display(), err))?;
            let span = Span {
                start: args.span_start,
                end: args.span_end,
            };
            let id = make_annotation_id(
                &args.artifact_uri,
                &span,
                &args.source_text,
                &args.target_text,
            );
            let annotation = Annotation {
                id,
                artifact_uri: args.artifact_uri,
                span,
                source_text: args.source_text,
                target_text: args.target_text,
                path_description: args
                    .path_description
                    .unwrap_or_else(|| "manual annotation".to_string()),
                invariant_type: InvariantType::from(args.invariant_type),
                preserved: args.preserved,
                losses: args.losses,
                break_condition: args.break_condition,
                classification: Classification::from(args.classification),
                visibility: Visibility::from(args.visibility),
                status: Status::from(args.status),
                created_by: args.created_by,
                updated_at: now_rfc3339(),
                notes: args.notes,
            };

            store
                .save(&annotation)
                .map_err(|err| format!("failed saving annotation: {}", err))?;

            if args.json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&annotation)
                        .map_err(|err| format!("failed to serialize annotation: {}", err))?
                );
            } else {
                println!("saved annotation {}", annotation.id);
            }
            Ok(())
        }
        AnnotationCommand::Accept(args) => {
            let store = AnnotationStore::new(&args.store)
                .map_err(|err| format!("failed opening store {}: {}", args.store.display(), err))?;
            let updated = store
                .update_status(&args.id, Status::Accepted, args.notes)
                .map_err(|err| format!("failed to accept annotation {}: {}", args.id, err))?;
            print_annotation_output(&updated, args.json)
        }
        AnnotationCommand::Dismiss(args) => {
            let store = AnnotationStore::new(&args.store)
                .map_err(|err| format!("failed opening store {}: {}", args.store.display(), err))?;
            let updated = store
                .dismiss_for_self(&args.id, args.notes)
                .map_err(|err| format!("failed to dismiss annotation {}: {}", args.id, err))?;
            print_annotation_output(&updated, args.json)
        }
        AnnotationCommand::Clarify(args) => {
            let store = AnnotationStore::new(&args.store)
                .map_err(|err| format!("failed opening store {}: {}", args.store.display(), err))?;
            let updated = store
                .update_status(&args.id, Status::Clarified, args.notes)
                .map_err(|err| format!("failed to clarify annotation {}: {}", args.id, err))?;
            print_annotation_output(&updated, args.json)
        }
        AnnotationCommand::Update(args) => {
            let store = AnnotationStore::new(&args.store)
                .map_err(|err| format!("failed opening store {}: {}", args.store.display(), err))?;
            let mut ann = store
                .get(&args.id)
                .map_err(|err| format!("failed loading annotation {}: {}", args.id, err))?;

            if let Some(invariant_type) = args.invariant_type {
                ann.invariant_type = InvariantType::from(invariant_type);
            }
            if let Some(classification) = args.classification {
                ann.classification = Classification::from(classification);
            }
            if let Some(visibility) = args.visibility {
                ann.visibility = Visibility::from(visibility);
            }
            if let Some(status) = args.status {
                ann.status = Status::from(status);
            }
            if let Some(preserved) = args.preserved {
                ann.preserved = parse_preserved(&preserved)?;
            }
            if let Some(break_condition) = args.break_condition {
                ann.break_condition = Some(break_condition);
            }
            for loss in args.add_losses {
                if !ann.losses.iter().any(|entry| entry == &loss) {
                    ann.losses.push(loss);
                }
            }
            if let Some(notes) = args.notes {
                if ann.notes.is_empty() {
                    ann.notes = notes;
                } else {
                    ann.notes = format!("{}\n{}", ann.notes, notes);
                }
            }
            ann.updated_at = now_rfc3339();

            store
                .update_annotation(&ann)
                .map_err(|err| format!("failed updating annotation {}: {}", args.id, err))?;

            print_annotation_output(&ann, args.json)
        }
    }
}

fn run_overlay(command: OverlayCommand) -> Result<(), String> {
    match command {
        OverlayCommand::Status(args) => {
            let store = AnnotationStore::new(&args.store)
                .map_err(|err| format!("failed opening store {}: {}", args.store.display(), err))?;
            let state = store
                .overlay_state()
                .map_err(|err| format!("failed reading overlay state: {}", err))?;
            if args.json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&state)
                        .map_err(|err| format!("failed to serialize overlay state: {}", err))?
                );
            } else {
                println!(
                    "overlay_enabled={} updated_at={}",
                    state.enabled, state.updated_at
                );
            }
            Ok(())
        }
        OverlayCommand::Toggle(args) => {
            let store = AnnotationStore::new(&args.store)
                .map_err(|err| format!("failed opening store {}: {}", args.store.display(), err))?;
            let enabled = matches!(args.state, ToggleArg::On);
            let state = store
                .set_overlay_state(enabled)
                .map_err(|err| format!("failed updating overlay state: {}", err))?;
            if args.json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&state)
                        .map_err(|err| format!("failed to serialize overlay state: {}", err))?
                );
            } else {
                println!(
                    "overlay_enabled={} updated_at={}",
                    state.enabled, state.updated_at
                );
            }
            Ok(())
        }
    }
}

fn parse_preserved(raw: &str) -> Result<Option<bool>, String> {
    match raw.to_ascii_lowercase().as_str() {
        "true" => Ok(Some(true)),
        "false" => Ok(Some(false)),
        "none" | "null" => Ok(None),
        _ => Err("--preserved must be one of: true, false, none".to_string()),
    }
}

fn path_matches_profile(artifact_uri: &str, suffixes: &[&str]) -> bool {
    suffixes.iter().any(|suffix| artifact_uri.ends_with(suffix))
}

fn profile_config(profile: ProfilePreset) -> ProfileConfig {
    match profile {
        ProfilePreset::Generic => ProfileConfig {
            name: "generic",
            default_visibility: Visibility::Private,
            include_suffixes: &[".md", ".markdown", ".adoc", ".txt"],
            focus: &["claim transitions"],
        },
        ProfilePreset::Echidna => ProfileConfig {
            name: "echidna",
            default_visibility: Visibility::SharedTeam,
            include_suffixes: &[".md", ".adoc", ".txt"],
            focus: &[
                "theorem -> guarantee",
                "assumption-bound -> general truth",
                "local result -> universal claim",
            ],
        },
        ProfilePreset::Panll => ProfileConfig {
            name: "panll",
            default_visibility: Visibility::Private,
            include_suffixes: &[".md", ".adoc", ".txt"],
            focus: &[
                "model -> reality",
                "benchmark -> capability",
                "descriptive -> normative",
            ],
        },
        ProfilePreset::Hypatia => ProfileConfig {
            name: "hypatia",
            default_visibility: Visibility::SharedTeam,
            include_suffixes: &[".md", ".adoc", ".txt"],
            focus: &[
                "data -> policy",
                "probability -> certainty",
                "benchmark -> capability",
            ],
        },
    }
}

fn print_annotation_output(annotation: &Annotation, json: bool) -> Result<(), String> {
    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(annotation)
                .map_err(|err| format!("failed to serialize annotation: {}", err))?
        );
    } else {
        println!(
            "annotation={} status={} visibility={:?}",
            annotation.id,
            enum_label_status(&annotation.status),
            annotation.visibility
        );
    }
    Ok(())
}

fn enum_label_status(value: &Status) -> &'static str {
    match value {
        Status::Open => "open",
        Status::Accepted => "accepted",
        Status::Dismissed => "dismissed",
        Status::Clarified => "clarified",
    }
}

fn enum_label_classification(value: &Classification) -> &'static str {
    match value {
        Classification::ValidPath => "valid_path",
        Classification::Overextended => "overextended",
        Classification::Conflation => "conflation",
        Classification::Incomplete => "incomplete",
        Classification::Abstain => "abstain",
    }
}

fn enum_label_invariant(value: &InvariantType) -> &'static str {
    match value {
        InvariantType::CausalRelationship => "causal_relationship",
        InvariantType::StatisticalRelationship => "statistical_relationship",
        InvariantType::LogicalImplication => "logical_implication",
        InvariantType::Mechanism => "mechanism",
        InvariantType::RiskProbability => "risk_probability",
        InvariantType::NormativeBridge => "normative_bridge",
        InvariantType::ResourceBudget => "resource_budget",
        InvariantType::Provenance => "provenance",
    }
}

impl From<VisibilityArg> for Visibility {
    fn from(value: VisibilityArg) -> Self {
        match value {
            VisibilityArg::Private => Visibility::Private,
            VisibilityArg::SharedTeam => Visibility::SharedTeam,
        }
    }
}

impl From<StatusArg> for Status {
    fn from(value: StatusArg) -> Self {
        match value {
            StatusArg::Open => Status::Open,
            StatusArg::Accepted => Status::Accepted,
            StatusArg::Dismissed => Status::Dismissed,
            StatusArg::Clarified => Status::Clarified,
        }
    }
}

impl From<InvariantTypeArg> for InvariantType {
    fn from(value: InvariantTypeArg) -> Self {
        match value {
            InvariantTypeArg::CausalRelationship => InvariantType::CausalRelationship,
            InvariantTypeArg::StatisticalRelationship => InvariantType::StatisticalRelationship,
            InvariantTypeArg::LogicalImplication => InvariantType::LogicalImplication,
            InvariantTypeArg::Mechanism => InvariantType::Mechanism,
            InvariantTypeArg::RiskProbability => InvariantType::RiskProbability,
            InvariantTypeArg::NormativeBridge => InvariantType::NormativeBridge,
            InvariantTypeArg::ResourceBudget => InvariantType::ResourceBudget,
            InvariantTypeArg::Provenance => InvariantType::Provenance,
        }
    }
}

impl From<ClassificationArg> for Classification {
    fn from(value: ClassificationArg) -> Self {
        match value {
            ClassificationArg::ValidPath => Classification::ValidPath,
            ClassificationArg::Overextended => Classification::Overextended,
            ClassificationArg::Conflation => Classification::Conflation,
            ClassificationArg::Incomplete => Classification::Incomplete,
            ClassificationArg::Abstain => Classification::Abstain,
        }
    }
}

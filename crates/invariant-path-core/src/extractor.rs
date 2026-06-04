// SPDX-License-Identifier: MPL-2.0
// Copyright (c) Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk>
#![forbid(unsafe_code)]

use crate::model::{ClaimCandidate, Span};

pub const TRIGGERS: [&str; 13] = [
    "therefore",
    "this proves",
    "this shows",
    "implies",
    "proves",
    "shows",
    "means",
    "must",
    "guarantees",
    "under assumptions",
    "in practice",
    "clearly",
    "obviously",
];

pub fn extract_candidates(text: &str) -> Vec<ClaimCandidate> {
    let sentence_spans = split_sentence_spans(text);
    let mut out = Vec::new();

    for (idx, (start, end)) in sentence_spans.iter().enumerate() {
        let raw_sentence = &text[*start..*end];
        let sentence = raw_sentence.trim();
        if sentence.len() < 12 {
            continue;
        }

        let lower = sentence.to_lowercase();
        let Some((trigger, trigger_idx)) = earliest_trigger(&lower) else {
            continue;
        };

        let mut source_text = sentence[..trigger_idx]
            .trim()
            .trim_matches(&[',', ';', ':', '.'][..])
            .to_string();
        let mut target_text = sentence[trigger_idx + trigger.len()..]
            .trim()
            .trim_matches(&[',', ';', ':', '.'][..])
            .to_string();

        if source_text.is_empty() && idx > 0 {
            let (prev_start, prev_end) = sentence_spans[idx - 1];
            source_text = text[prev_start..prev_end]
                .trim()
                .trim_matches(&[',', ';', ':', '.'][..])
                .to_string();
        }

        if source_text.is_empty() {
            if let Some((secondary_source, secondary_target)) =
                split_secondary_transition(&target_text)
            {
                source_text = secondary_source;
                target_text = secondary_target;
            }
        }

        if source_text.len() < 5 || target_text.len() < 5 {
            continue;
        }

        let leading_trim = raw_sentence
            .len()
            .saturating_sub(raw_sentence.trim_start().len());
        let span = Span {
            start: start + leading_trim,
            end: *end,
        };

        let path_description = format!(
            "{} -> {} via '{}'",
            compact(&source_text),
            compact(&target_text),
            trigger
        );

        out.push(ClaimCandidate {
            span,
            source_text,
            target_text,
            trigger: trigger.to_string(),
            path_description,
        });
    }

    out
}

fn compact(input: &str) -> String {
    let mut value = input.trim().replace('\n', " ");
    if value.len() > 90 {
        value.truncate(87);
        value.push_str("...");
    }
    value
}

fn earliest_trigger(lower_sentence: &str) -> Option<(&'static str, usize)> {
    let mut best: Option<(&'static str, usize)> = None;

    for trigger in TRIGGERS {
        if let Some(idx) = lower_sentence.find(trigger) {
            match best {
                Some((_, best_idx)) if best_idx <= idx => {}
                _ => best = Some((trigger, idx)),
            }
        }
    }

    best
}

fn split_secondary_transition(text: &str) -> Option<(String, String)> {
    let lower = text.to_lowercase();
    let tokens = [
        " implies ",
        " means ",
        " proves ",
        " shows ",
        " guarantees ",
        " must ",
    ];

    for token in tokens {
        if let Some(pos) = lower.find(token) {
            let source = text[..pos].trim().trim_matches(&[',', ';', ':', '.'][..]);
            let target = text[pos + token.len()..]
                .trim()
                .trim_matches(&[',', ';', ':', '.'][..]);
            if source.len() >= 5 && target.len() >= 5 {
                return Some((source.to_string(), target.to_string()));
            }
        }
    }

    None
}

fn split_sentence_spans(text: &str) -> Vec<(usize, usize)> {
    let mut spans = Vec::new();
    let mut start = 0usize;

    for (idx, ch) in text.char_indices() {
        if matches!(ch, '.' | '!' | '?' | '\n') {
            let end = idx + ch.len_utf8();
            if end > start {
                spans.push((start, end));
            }
            start = end;
        }
    }

    if start < text.len() {
        spans.push((start, text.len()));
    }

    spans
}

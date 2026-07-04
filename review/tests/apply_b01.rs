mod common;

use am001::apply::{
    apply_parsed_batch, load_staging, parse_batch_text, staging_path_for, trace_path_for,
    write_report, write_trace_file, ApplyAction, EventSource, RejectReason, StagingSidecar,
    STAGING_FORMAT_VERSION,
};
use am001::core::diff::trace_jsonl_bytes;
use am001::core::state::AmState;
use am001::core::step::step_result;
use am001::parser::rule::parse_rule_line;
use common::theta_default;
use std::collections::BTreeMap;
use std::fs;
use std::process::Command;
use tempfile::tempdir;

#[test]
fn grammar_accepts_and_rejects() {
    let accepted = parse_batch_text(&batch(&[
        r#"{"id":1,"session_id":"s","source":"user","verb":"assert","args":{"concept":"rust","axes":[{"axis":"truth_assert","value":1.0}]}}"#,
        r#"{"id":2,"session_id":"s","source":"test_verified","verb":"cue","args":{"concept":"rust","strength":0.8}}"#,
        r#"{"id":3,"session_id":"s","source":"user","verb":"link","args":{"a":"rust","b":"am001","weight":1.0}}"#,
        r#"{"id":4,"session_id":"s","source":"user","verb":"goal_push","args":{"concept":"rust"}}"#,
        r#"{"id":5,"session_id":"s","source":"user","verb":"goal_pop","args":{"concept":"rust"}}"#,
    ]))
    .unwrap();
    assert_eq!(accepted.events.len(), 5);
    assert!(accepted.rejections.is_empty());

    assert_eq!(
        parse_batch_text(r#"{"grammar":"EG-X"}"#)
            .unwrap_err()
            .reason,
        RejectReason::BadVersion
    );

    let cases = [
        (
            r#"{"id":1,"session_id":"s","source":"user","verb":"nonsense","args":{}}"#,
            RejectReason::UnknownVerb,
        ),
        (
            r#"{"id":1,"session_id":"s","source":"user","verb":"cue","extra":0,"args":{"concept":"rust","strength":0.8}}"#,
            RejectReason::UnknownField,
        ),
        (
            r#"{"id":1,"session_id":"s","source":"user","verb":"cue","args":{"concept":"rust","strength":0.8}}
{"id":1,"session_id":"s","source":"user","verb":"cue","args":{"concept":"rust","strength":0.8}}"#,
            RejectReason::NonMonotonicId,
        ),
        (
            r#"{"id":1,"session_id":"s","source":"user","verb":"assert","args":{"concept":"rust","axes":[{"axis":"truth_assert","value":"NaN"}]}}"#,
            RejectReason::NonFiniteValue,
        ),
        (
            r#"{"id":1,"session_id":"s","source":"user","verb":"assert","args":{"concept":"rust","axes":[{"axis":"certainty","value":1.0}]}}"#,
            RejectReason::UnknownAxis,
        ),
        (
            r#"{"id":1,"session_id":"s","source":"user","verb":"cue","args":{"concept":"rust","strength":1.2}}"#,
            RejectReason::StrengthOutOfRange,
        ),
        (
            r#"{"id":1,"session_id":"s","source":"user","verb":"link","args":{"a":"rust","b":"am001","weight":2.0}}"#,
            RejectReason::WeightOutOfRange,
        ),
    ];

    for (line, reason) in cases {
        let parsed = parse_batch_text(&batch(line.lines().collect::<Vec<_>>().as_slice())).unwrap();
        assert_eq!(parsed.rejections.len(), 1, "{line}");
        assert_eq!(parsed.rejections[0].reason, Some(reason), "{line}");
    }
}

#[test]
fn apply_matches_step_text() {
    let theta = theta_default();
    let mut apply_state = AmState::new(theta.clone()).unwrap();
    let mut staging = StagingSidecar::default();
    let parsed = parse_batch_text(&batch(&[
        r#"{"id":1,"session_id":"s","source":"user","verb":"assert","args":{"concept":"rust","axes":[{"axis":"truth_assert","value":1.0},{"axis":"goal_relevance","value":0.8}]}}"#,
        r#"{"id":2,"session_id":"s","source":"user","verb":"cue","args":{"concept":"rust","strength":0.8}}"#,
        r#"{"id":3,"session_id":"s","source":"user","verb":"link","args":{"a":"rust","b":"am001","weight":1.0}}"#,
    ]))
    .unwrap();
    let outcome = apply_parsed_batch(&mut apply_state, &parsed, &mut staging).unwrap();
    assert!(!outcome.report.has_rejections());

    let mut text_state = AmState::new(theta).unwrap();
    let mut text_traces = Vec::new();
    for command in [
        "assert rust truth_assert=1 goal_relevance=0.8",
        "cue rust 0.8",
        "link rust am001 1",
    ] {
        let event = parse_rule_line(command, text_state.tick + 1).unwrap();
        text_traces.push(step_result(&mut text_state, &event).unwrap());
    }

    assert_eq!(apply_state.snapshot_bytes(), text_state.snapshot_bytes());
    assert_eq!(
        trace_jsonl_bytes(&outcome.traces).unwrap(),
        trace_jsonl_bytes(&text_traces).unwrap()
    );
}

#[test]
fn staging_lifecycle() {
    let theta = theta_default();
    let mut state = AmState::new(theta).unwrap();
    let mut staging = StagingSidecar::default();

    let stage = parse_batch_text(&batch(&[r#"{"id":1,"session_id":"s","source":"llm_claim","verb":"assert","args":{"concept":"rust","axes":[{"axis":"truth_assert","value":1.0}]}}"#])).unwrap();
    let outcome = apply_parsed_batch(&mut state, &stage, &mut staging).unwrap();
    assert_eq!(outcome.report.summary.staged, 1);
    assert_eq!(outcome.traces.len(), 0);
    assert_eq!(state.tick, 0);
    assert_eq!(staging.entries.len(), 1);

    let contradict = parse_batch_text(&batch(&[r#"{"id":2,"session_id":"s","source":"user","verb":"assert","args":{"concept":"rust","axes":[{"axis":"truth_assert","value":-1.0}]}}"#])).unwrap();
    let outcome = apply_parsed_batch(&mut state, &contradict, &mut staging).unwrap();
    assert_eq!(outcome.report.summary.applied, 1);
    assert_eq!(outcome.report.summary.committed_from_staging, 0);
    assert_eq!(staging.entries.len(), 1);

    let corroborate = parse_batch_text(&batch(&[r#"{"id":3,"session_id":"s","source":"test_verified","verb":"assert","args":{"concept":"rust","axes":[{"axis":"truth_assert","value":1.0}]}}"#])).unwrap();
    let outcome = apply_parsed_batch(&mut state, &corroborate, &mut staging).unwrap();
    assert_eq!(outcome.report.summary.applied, 1);
    assert_eq!(outcome.report.summary.committed_from_staging, 1);
    assert!(staging.entries.is_empty());
    assert_eq!(
        outcome
            .traces
            .iter()
            .filter(|trace| trace.event_id == 1)
            .count(),
        1
    );

    let mut expiring_state = AmState::new(theta_default()).unwrap();
    let mut expiring = StagingSidecar::default();
    let mut lines = vec![r#"{"id":1,"session_id":"s","source":"llm_claim","verb":"link","args":{"a":"a","b":"b","weight":1.0}}"#.to_string()];
    for idx in 0..200 {
        lines.push(format!(
            r#"{{"id":{},"session_id":"s","source":"user","verb":"cue","args":{{"concept":"clock","strength":0.1}}}}"#,
            idx + 2
        ));
    }
    let refs = lines.iter().map(String::as_str).collect::<Vec<_>>();
    let parsed = parse_batch_text(&batch(&refs)).unwrap();
    let outcome = apply_parsed_batch(&mut expiring_state, &parsed, &mut expiring).unwrap();
    assert_eq!(outcome.report.summary.staged, 1);
    assert_eq!(outcome.report.summary.expired, 1);
    assert!(expiring.entries.is_empty());
    assert_eq!(expiring.tombstones.len(), 1);
}

#[test]
fn staging_roundtrip_byte_exact_and_refuses_corruption() {
    let parsed = parse_batch_text(&batch(&[r#"{"id":1,"session_id":"s","source":"llm_claim","verb":"link","args":{"a":"a","b":"b","weight":1.0}}"#])).unwrap();
    let mut state = AmState::new(theta_default()).unwrap();
    let mut sidecar = StagingSidecar::default();
    apply_parsed_batch(&mut state, &parsed, &mut sidecar).unwrap();

    let left = sidecar.to_bytes().unwrap();
    let right = StagingSidecar::from_bytes(&left)
        .unwrap()
        .to_bytes()
        .unwrap();
    assert_eq!(left, right);

    let dir = tempdir().unwrap();
    let corrupt = dir.path().join("bad.staging");
    fs::write(&corrupt, b"not-json").unwrap();
    assert!(load_staging(&corrupt)
        .unwrap_err()
        .to_string()
        .contains("staging sidecar corrupt"));

    let mismatch = dir.path().join("mismatch.staging");
    fs::write(
        &mismatch,
        format!(
            r#"{{"format_version":{},"grammar":"EG-1","applied_event_count":0,"entries":[],"tombstones":[]}}"#,
            STAGING_FORMAT_VERSION + 1
        ),
    )
    .unwrap();
    assert!(load_staging(&mismatch)
        .unwrap_err()
        .to_string()
        .contains("format_version"));
}

#[test]
fn provenance_joinable() {
    let parsed = parse_batch_text(&batch(&[
        r#"{"id":1,"session_id":"s","source":"user","verb":"assert","args":{"concept":"alpha","axes":[{"axis":"truth_assert","value":1.0}]}}"#,
        r#"{"id":2,"session_id":"s","source":"test_verified","verb":"assert","args":{"concept":"beta","axes":[{"axis":"truth_assert","value":1.0}]}}"#,
        r#"{"id":3,"session_id":"s","source":"llm_claim","verb":"assert","args":{"concept":"gamma","axes":[{"axis":"truth_assert","value":1.0}]}}"#,
        r#"{"id":4,"session_id":"s","source":"user","verb":"assert","args":{"concept":"gamma","axes":[{"axis":"truth_assert","value":1.0}]}}"#,
    ]))
    .unwrap();
    let mut state = AmState::new(theta_default()).unwrap();
    let mut staging = StagingSidecar::default();
    let outcome = apply_parsed_batch(&mut state, &parsed, &mut staging).unwrap();

    let mut by_id = BTreeMap::new();
    for verdict in &outcome.report.events {
        if matches!(
            verdict.action,
            ApplyAction::Applied | ApplyAction::CommittedFromStaging
        ) {
            by_id.insert(
                verdict.id.unwrap(),
                (verdict.source.unwrap(), verdict.action),
            );
        }
    }

    assert_eq!(by_id.get(&1).unwrap().0, EventSource::User);
    assert_eq!(by_id.get(&2).unwrap().0, EventSource::TestVerified);
    assert_eq!(by_id.get(&3).unwrap().0, EventSource::LlmClaim);
    assert_eq!(by_id.get(&3).unwrap().1, ApplyAction::CommittedFromStaging);

    for trace in outcome
        .traces
        .iter()
        .filter(|trace| !trace.mutations.is_empty())
    {
        assert!(by_id.contains_key(&(trace.event_id as u64)));
    }
}

#[test]
fn same_snapshot_and_events_are_byte_deterministic() {
    let text = batch(&[
        r#"{"id":1,"session_id":"s","source":"llm_claim","verb":"assert","args":{"concept":"rust","axes":[{"axis":"truth_assert","value":1.0}]}}"#,
        r#"{"id":2,"session_id":"s","source":"user","verb":"assert","args":{"concept":"rust","axes":[{"axis":"truth_assert","value":1.0}]}}"#,
        r#"{"id":3,"session_id":"s","source":"test_verified","verb":"cue","args":{"concept":"rust","strength":0.8}}"#,
    ]);
    let parsed = parse_batch_text(&text).unwrap();

    let run = || {
        let mut state = AmState::new(theta_default()).unwrap();
        let mut sidecar = StagingSidecar::default();
        let outcome = apply_parsed_batch(&mut state, &parsed, &mut sidecar).unwrap();
        (
            state.snapshot_bytes(),
            trace_jsonl_bytes(&outcome.traces).unwrap(),
            serde_json::to_vec(&outcome.report).unwrap(),
            sidecar.to_bytes().unwrap(),
        )
    };

    assert_eq!(run(), run());
}

#[test]
fn fuzz_rejection_never_mutates() {
    let mut seed = 0xB01_u64;
    for _ in 0..24 {
        seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
        let bad_line = match seed % 4 {
            0 => r#"{"id":1,"session_id":"s","source":"user","verb":"cue","args":{"concept":"rust","strength":0.8},"extra":1}"#.to_string(),
            1 => r#"{"id":1,"session_id":"s","source":"user","verb":"unknown","args":{}}"#.to_string(),
            2 => r#"{"id":1,"session_id":"s","source":"bad","verb":"cue","args":{"concept":"rust","strength":0.8}}"#.to_string(),
            _ => format!(
                r#"{{"id":1,"session_id":"s","source":"user","verb":"cue","args":{{"concept":"rust","strength":0.8}}}}
{{"id":{},"session_id":"s","source":"user","verb":"cue","args":{{"concept":"rust","strength":0.8}}}}"#,
                1
            ),
        };
        let parsed = parse_batch_text(&batch(&bad_line.lines().collect::<Vec<_>>())).unwrap();
        assert!(parsed.has_structural_rejection());
        let mut state = AmState::new(theta_default()).unwrap();
        let before = state.snapshot_bytes();
        let mut staging = StagingSidecar::default();
        let outcome = apply_parsed_batch(&mut state, &parsed, &mut staging).unwrap();
        assert_eq!(outcome.traces.len(), 0);
        assert_eq!(state.snapshot_bytes(), before);
    }

    let text = batch(&[
        r#"{"id":1,"session_id":"s","source":"user","verb":"assert","args":{"concept":"a","axes":[{"axis":"truth_assert","value":1.0}]}}"#,
        r#"{"id":2,"session_id":"s","source":"user","verb":"assert","args":{"concept":"bad","axes":[{"axis":"certainty","value":1.0}]}}"#,
        r#"{"id":3,"session_id":"s","source":"user","verb":"assert","args":{"concept":"b","axes":[{"axis":"truth_assert","value":1.0}]}}"#,
    ]);
    let parsed = parse_batch_text(&text).unwrap();
    assert!(!parsed.has_structural_rejection());
    assert_eq!(parsed.rejections[0].reason, Some(RejectReason::UnknownAxis));

    let mut semantic_state = AmState::new(theta_default()).unwrap();
    let mut staging = StagingSidecar::default();
    apply_parsed_batch(&mut semantic_state, &parsed, &mut staging).unwrap();

    let mut expected = AmState::new(theta_default()).unwrap();
    for (id, line) in [
        (1, "assert a truth_assert=1"),
        (3, "assert b truth_assert=1"),
    ] {
        let event = parse_rule_line(line, id).unwrap();
        step_result(&mut expected, &event).unwrap();
    }
    assert_eq!(semantic_state.snapshot_bytes(), expected.snapshot_bytes());
}

#[test]
fn cli_apply_writes_report_trace_and_staging() {
    let dir = tempdir().unwrap();
    let snapshot = dir.path().join("am.bin");
    let events = dir.path().join("events.jsonl");
    let report = dir.path().join("report.json");
    fs::write(
        &events,
        batch(&[r#"{"id":1,"session_id":"s","source":"user","verb":"assert","args":{"concept":"rust","axes":[{"axis":"truth_assert","value":1.0}]}}"#]),
    )
    .unwrap();

    let status = Command::new(env!("CARGO_BIN_EXE_am"))
        .args(["apply", "--snapshot"])
        .arg(&snapshot)
        .args(["--events"])
        .arg(&events)
        .args(["--report"])
        .arg(&report)
        .status()
        .unwrap();
    assert!(status.success());
    assert!(snapshot.exists());
    assert!(report.exists());
    assert!(staging_path_for(&snapshot).exists());
    assert!(trace_path_for(&snapshot).exists());
    let report_text = fs::read_to_string(&report).unwrap();
    assert!(report_text.contains("\"action\": \"applied\""));
}

#[test]
fn report_and_trace_serialization_are_stable() {
    let parsed = parse_batch_text(&batch(&[r#"{"id":1,"session_id":"s","source":"user","verb":"assert","args":{"concept":"rust","axes":[{"axis":"truth_assert","value":1.0}]}}"#])).unwrap();
    let mut state = AmState::new(theta_default()).unwrap();
    let mut staging = StagingSidecar::default();
    let outcome = apply_parsed_batch(&mut state, &parsed, &mut staging).unwrap();

    let dir = tempdir().unwrap();
    let report_path = dir.path().join("report.json");
    let trace_path = dir.path().join("trace.jsonl");
    write_report(&report_path, &outcome.report).unwrap();
    write_trace_file(&trace_path, &outcome.traces).unwrap();
    assert_eq!(
        fs::read(&trace_path).unwrap(),
        trace_jsonl_bytes(&outcome.traces).unwrap()
    );
    assert_eq!(
        fs::read(&report_path).unwrap(),
        serde_json::to_vec_pretty(&outcome.report).unwrap()
    );
}

fn batch(lines: &[&str]) -> String {
    let mut out = String::from(r#"{"grammar":"EG-1"}"#);
    out.push('\n');
    for line in lines {
        out.push_str(line);
        out.push('\n');
    }
    out
}

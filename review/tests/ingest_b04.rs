use am001::apply::{load_staging, parse_batch_text, staging_path_for, ApplyAction, EventSource};
use am001::core::axes::axis_index;
use am001::core::event::ConceptRef;
use am001::storage::snapshot_file::load_snapshot;
use serde_json::Value;
use std::fs;
use std::process::Command;
use tempfile::tempdir;

#[test]
fn ingest_all_kinds_emit_valid_test_verified_events() {
    let dir = tempdir().unwrap();
    let cases = [
        (
            "cargo-test",
            "tests/fixtures/b04/cargo_test.txt",
            "test_suite",
            2,
        ),
        (
            "sweep-csv",
            "tests/fixtures/b04/sweep.csv",
            "sweep_abcdef12",
            1,
        ),
        (
            "world-run",
            "tests/fixtures/b04/world_run.txt",
            "world_run",
            1,
        ),
    ];

    for (kind, fixture, concept, expected_events) in cases {
        let events = dir.path().join(format!("{kind}.jsonl"));
        let output = Command::new(env!("CARGO_BIN_EXE_am"))
            .args([
                "ingest",
                "--kind",
                kind,
                "--input",
                fixture,
                "--session",
                "s_b04",
                "--events-out",
                events.to_str().unwrap(),
            ])
            .output()
            .unwrap();
        assert!(output.status.success(), "{}", stderr(&output));
        let text = fs::read_to_string(&events).unwrap();
        let parsed = parse_batch_text(&text).unwrap();
        assert!(parsed.rejections.is_empty());
        assert_eq!(parsed.events.len(), expected_events);
        for event in &parsed.events {
            assert_eq!(event.source, EventSource::TestVerified);
            assert_eq!(event.session_id, "s_b04");
            match &event.args {
                am001::apply::EgArgs::Assert {
                    concept: label,
                    axes,
                } => {
                    assert_eq!(label, concept);
                    for axis in axes {
                        assert!(
                            (-1.0..=1.0).contains(&axis.value),
                            "{} value {} outside [-1,1]",
                            axis.axis,
                            axis.value
                        );
                    }
                }
                other => panic!("unexpected ingest args: {other:?}"),
            }
        }
        assert!(
            !text.contains("raw_name_is_not_stored"),
            "raw cargo-test text leaked into EG-1"
        );
    }
}

#[test]
fn ingest_rejects_malformed_inputs_without_partial_output() {
    let dir = tempdir().unwrap();
    for (kind, fixture) in [
        ("cargo-test", "tests/fixtures/b04/cargo_bad.txt"),
        ("sweep-csv", "tests/fixtures/b04/sweep_bad.csv"),
        ("world-run", "tests/fixtures/b04/world_run_bad.txt"),
    ] {
        let events = dir.path().join(format!("{kind}.jsonl"));
        let output = Command::new(env!("CARGO_BIN_EXE_am"))
            .args([
                "ingest",
                "--kind",
                kind,
                "--input",
                fixture,
                "--session",
                "s_bad",
                "--events-out",
                events.to_str().unwrap(),
            ])
            .output()
            .unwrap();
        assert!(!output.status.success(), "{kind}");
        assert!(!events.exists(), "{kind} wrote partial output");
        assert!(stderr(&output).starts_with("Error: ingest rejected:"));
    }
}

#[test]
fn distill_extracts_one_fence_forces_source_and_rejects_bad_inputs() {
    let dir = tempdir().unwrap();
    let events = dir.path().join("claims.jsonl");
    let output = Command::new(env!("CARGO_BIN_EXE_am"))
        .args([
            "distill",
            "--input",
            "tests/fixtures/b04/distill_valid.txt",
            "--session",
            "s_distill",
            "--events-out",
            events.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(output.status.success(), "{}", stderr(&output));
    let text = fs::read_to_string(&events).unwrap();
    let parsed = parse_batch_text(&text).unwrap();
    assert_eq!(parsed.events.len(), 1);
    assert_eq!(parsed.events[0].source, EventSource::LlmClaim);
    assert_eq!(parsed.events[0].session_id, "s_distill");

    for fixture in [
        "tests/fixtures/b04/distill_missing.txt",
        "tests/fixtures/b04/distill_dual.txt",
        "tests/fixtures/b04/distill_invalid.txt",
    ] {
        let bad_out = dir
            .path()
            .join(format!("{}.jsonl", fixture.replace('/', "_")));
        let output = Command::new(env!("CARGO_BIN_EXE_am"))
            .args([
                "distill",
                "--input",
                fixture,
                "--session",
                "s_distill",
                "--events-out",
                bad_out.to_str().unwrap(),
            ])
            .output()
            .unwrap();
        assert!(!output.status.success(), "{fixture}");
        assert!(!bad_out.exists(), "{fixture} wrote a partial output");
        assert!(stderr(&output).starts_with("Error: distill rejected:"));
    }
}

#[test]
fn distilled_claim_stages_and_ingested_test_verified_event_commits_it() {
    let dir = tempdir().unwrap();
    let snapshot = dir.path().join("am.bin");
    let llm_input = dir.path().join("llm.txt");
    let llm_events = dir.path().join("llm.jsonl");
    let test_input = dir.path().join("cargo.txt");
    let test_events = dir.path().join("test.jsonl");
    let report1 = dir.path().join("r1.json");
    let report2 = dir.path().join("r2.json");

    run_ok(Command::new(env!("CARGO_BIN_EXE_am")).args([
        "init",
        "--snapshot",
        snapshot.to_str().unwrap(),
    ]));
    fs::write(
        &llm_input,
        "```eg1\n{\"id\":1,\"verb\":\"assert\",\"args\":{\"concept\":\"test_suite\",\"axes\":[{\"axis\":\"truth_assert\",\"value\":1.0}]}}\n```\n",
    )
    .unwrap();
    run_ok(Command::new(env!("CARGO_BIN_EXE_am")).args([
        "distill",
        "--input",
        llm_input.to_str().unwrap(),
        "--session",
        "s_commit",
        "--events-out",
        llm_events.to_str().unwrap(),
    ]));
    run_ok(Command::new(env!("CARGO_BIN_EXE_am")).args([
        "apply",
        "--snapshot",
        snapshot.to_str().unwrap(),
        "--events",
        llm_events.to_str().unwrap(),
        "--report",
        report1.to_str().unwrap(),
    ]));
    let first = read_report(&report1);
    assert_eq!(first["summary"]["staged"], 1);
    assert_eq!(first["summary"]["applied"], 0);

    fs::write(
        &test_input,
        "test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out\n",
    )
    .unwrap();
    run_ok(Command::new(env!("CARGO_BIN_EXE_am")).args([
        "ingest",
        "--kind",
        "cargo-test",
        "--input",
        test_input.to_str().unwrap(),
        "--session",
        "s_commit",
        "--events-out",
        test_events.to_str().unwrap(),
    ]));
    run_ok(Command::new(env!("CARGO_BIN_EXE_am")).args([
        "apply",
        "--snapshot",
        snapshot.to_str().unwrap(),
        "--events",
        test_events.to_str().unwrap(),
        "--report",
        report2.to_str().unwrap(),
    ]));
    let second = read_report(&report2);
    assert_eq!(second["summary"]["applied"], 1);
    assert_eq!(second["summary"]["committed_from_staging"], 1);
    let committed = second["events"]
        .as_array()
        .unwrap()
        .iter()
        .any(|event| event["action"] == serde_json::json!(ApplyAction::CommittedFromStaging));
    assert!(committed);
    let staging = load_staging(&staging_path_for(&snapshot)).unwrap();
    assert!(staging.entries.is_empty());
    let state = load_snapshot(&snapshot).unwrap();
    let row = state
        .resolve_existing(&ConceptRef::Label("test_suite".to_string()))
        .unwrap();
    assert!(state.m_get(row, axis_index("truth_assert").unwrap()) > 0.0);
}

#[test]
fn ingest_and_distill_outputs_are_deterministic() {
    let dir = tempdir().unwrap();
    let left = dir.path().join("left.jsonl");
    let right = dir.path().join("right.jsonl");
    for out in [&left, &right] {
        run_ok(Command::new(env!("CARGO_BIN_EXE_am")).args([
            "ingest",
            "--kind",
            "sweep-csv",
            "--input",
            "tests/fixtures/b04/sweep.csv",
            "--session",
            "s_det",
            "--events-out",
            out.to_str().unwrap(),
        ]));
    }
    assert_eq!(fs::read(&left).unwrap(), fs::read(&right).unwrap());
}

fn run_ok(command: &mut Command) {
    let output = command.output().unwrap();
    assert!(output.status.success(), "{}", stderr(&output));
}

fn read_report(path: &std::path::Path) -> Value {
    serde_json::from_str(&fs::read_to_string(path).unwrap()).unwrap()
}

fn stderr(output: &std::process::Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

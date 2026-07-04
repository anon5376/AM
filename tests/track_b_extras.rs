use am001::apply::{staging_path_for, trace_path_for};
use serde_json::Value;
use std::fs;
use std::process::Command;
use tempfile::tempdir;

#[test]
fn apply_dry_run_reports_without_writing_state_sidecars_or_trace() {
    let dir = tempdir().unwrap();
    let snapshot = dir.path().join("am.bin");
    let events = dir.path().join("events.jsonl");
    let report = dir.path().join("report.json");
    fs::write(
        &events,
        batch(&[r#"{"id":1,"session_id":"s","source":"user","verb":"assert","args":{"concept":"cpt_1","axes":[{"axis":"truth_assert","value":1.0}]}}"#]),
    )
    .unwrap();

    run_ok(Command::new(env!("CARGO_BIN_EXE_am")).args([
        "apply",
        "--dry-run",
        "--snapshot",
        snapshot.to_str().unwrap(),
        "--events",
        events.to_str().unwrap(),
        "--report",
        report.to_str().unwrap(),
    ]));

    assert!(!snapshot.exists());
    assert!(!staging_path_for(&snapshot).exists());
    assert!(!trace_path_for(&snapshot).exists());
    let report_json = read_json(&report);
    assert_eq!(report_json["summary"]["applied"], 1);
    assert!(report_json.get("trace_file").is_none());
    assert!(report_json.get("staging_file").is_none());
}

#[test]
fn provenance_summarizes_apply_trace_by_event_id() {
    let dir = tempdir().unwrap();
    let snapshot = dir.path().join("am.bin");
    let events = dir.path().join("events.jsonl");
    fs::write(
        &events,
        batch(&[r#"{"id":1,"session_id":"s","source":"user","verb":"assert","args":{"concept":"cpt_1","axes":[{"axis":"truth_assert","value":1.0}]}}"#]),
    )
    .unwrap();
    run_ok(Command::new(env!("CARGO_BIN_EXE_am")).args([
        "apply",
        "--snapshot",
        snapshot.to_str().unwrap(),
        "--events",
        events.to_str().unwrap(),
    ]));

    let output = Command::new(env!("CARGO_BIN_EXE_am"))
        .args([
            "provenance",
            "--snapshot",
            snapshot.to_str().unwrap(),
            "--event",
            "1",
        ])
        .output()
        .unwrap();
    assert!(output.status.success(), "{}", stderr(&output));
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("event 1\n"));
    assert!(stdout.contains("traces 1\n"));
    assert!(stdout.contains("mutations "));
    assert!(stdout.contains("causes:\n"));
    assert!(stdout.contains("targets:\n"));
}

fn run_ok(command: &mut Command) {
    let output = command.output().unwrap();
    assert!(output.status.success(), "{}", stderr(&output));
}

fn read_json(path: &std::path::Path) -> Value {
    serde_json::from_str(&fs::read_to_string(path).unwrap()).unwrap()
}

fn stderr(output: &std::process::Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
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

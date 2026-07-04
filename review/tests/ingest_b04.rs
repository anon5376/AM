use am001::apply::{load_staging, parse_batch_text, staging_path_for, ApplyAction, EventSource};
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
            "running 1 test\ntest raw_name_is_not_stored ... ok\n\ntest result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out\n",
        ),
        (
            "sweep-csv",
            "theta_hash,all_pass,completion_recall,completion_contamination\nabc,false,0.2,0.3\ndef,true,1.0,0.0\n",
        ),
        (
            "world-run",
            "world-run actions=4 ran=4 termination=goal obs=/tmp/obs.jsonl trace=/tmp/trace.jsonl\n",
        ),
    ];

    for (kind, input_text) in cases {
        let input = dir.path().join(format!("{kind}.txt"));
        let events = dir.path().join(format!("{kind}.jsonl"));
        fs::write(&input, input_text).unwrap();
        let output = Command::new(env!("CARGO_BIN_EXE_am"))
            .args([
                "ingest",
                "--kind",
                kind,
                "--input",
                input.to_str().unwrap(),
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
        assert_eq!(parsed.events.len(), 1);
        assert_eq!(parsed.events[0].source, EventSource::TestVerified);
        assert_eq!(parsed.events[0].session_id, "s_b04");
        assert!(
            !text.contains("raw_name_is_not_stored"),
            "raw cargo-test text leaked into EG-1"
        );
    }
}

#[test]
fn distill_extracts_one_fence_forces_source_and_rejects_bad_inputs() {
    let dir = tempdir().unwrap();
    let input = dir.path().join("llm.txt");
    let events = dir.path().join("claims.jsonl");
    fs::write(
        &input,
        r#"ignored prose
```eg1
{"id":7,"session_id":"wrong","source":"user","verb":"assert","args":{"concept":"cpt_1","axes":[{"axis":"truth_assert","value":1.0}]}}
```
"#,
    )
    .unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_am"))
        .args([
            "distill",
            "--input",
            input.to_str().unwrap(),
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

    for (name, body) in [
        ("missing.txt", "no fenced block"),
        (
            "dual.txt",
            "```eg1\n{\"id\":1,\"verb\":\"cue\",\"args\":{\"concept\":\"cpt_1\",\"strength\":0.8}}\n```\n```eg1\n{\"id\":2,\"verb\":\"cue\",\"args\":{\"concept\":\"cpt_2\",\"strength\":0.8}}\n```",
        ),
        (
            "raw_label.txt",
            "```eg1\n{\"id\":1,\"verb\":\"assert\",\"args\":{\"concept\":\"rust\",\"axes\":[{\"axis\":\"truth_assert\",\"value\":1.0}]}}\n```",
        ),
    ] {
        let bad_input = dir.path().join(name);
        let bad_out = dir.path().join(format!("{name}.jsonl"));
        fs::write(&bad_input, body).unwrap();
        let output = Command::new(env!("CARGO_BIN_EXE_am"))
            .args([
                "distill",
                "--input",
                bad_input.to_str().unwrap(),
                "--session",
                "s_distill",
                "--events-out",
                bad_out.to_str().unwrap(),
            ])
            .output()
            .unwrap();
        assert!(!output.status.success(), "{name}");
        assert!(!bad_out.exists(), "{name} wrote a partial output");
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
        "```eg1\n{\"id\":1,\"verb\":\"assert\",\"args\":{\"concept\":\"cpt_1\",\"axes\":[{\"axis\":\"truth_assert\",\"value\":1.0}]}}\n```\n",
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

use am001::apply::trace_path_for;
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::tempdir;

const GOLDEN: &str = include_str!("golden/b05_dashboard.html");

#[test]
fn dashboard_cli_matches_golden_and_is_self_contained() {
    let dir = tempdir().unwrap();
    let (snapshot, trace, report) = build_dashboard_inputs(dir.path());
    let out1 = dir.path().join("dash1.html");
    let out2 = dir.path().join("dash2.html");

    run_dashboard(&snapshot, &trace, &report, &out1);
    run_dashboard(&snapshot, &trace, &report, &out2);
    let html1 = fs::read_to_string(&out1).unwrap();
    let html2 = fs::read_to_string(&out2).unwrap();
    assert_eq!(html1, html2);
    assert_eq!(html1, GOLDEN);
    assert!(html1.contains("<script id=\"am-data\" type=\"application/json\">"));
    assert!(!html1.contains("http://"));
    assert!(!html1.contains("https://"));
    assert!(!html1.contains("<script src="));
    assert!(!html1.contains("<link "));
    assert!(!html1.contains("localhost"));
    for anchor in [
        "compiled-context",
        "contradictions",
        "staging",
        "beval",
        "provenance",
    ] {
        assert!(html1.contains(anchor), "missing dashboard anchor {anchor}");
    }
}

#[test]
fn dashboard_embedded_json_smoke() {
    let data = embedded_payload(GOLDEN);
    assert_eq!(data["schema"], "AM-DASHBOARD-1");
    assert_eq!(data["snapshot"]["row_count"], 2);
    assert!(data["compiled_context"]
        .as_str()
        .unwrap()
        .contains("STATE:"));
    assert_eq!(
        data["snapshot"]["contradictions"].as_array().unwrap().len(),
        1
    );
    assert_eq!(data["staging"]["entries"].as_array().unwrap().len(), 1);
    assert_eq!(data["beval"].as_array().unwrap().len(), 1);
    assert_eq!(data["beval"][0]["lane"], "b2");
    assert_eq!(data["beval"][0]["total_context_tokens"], 15457);
    assert_eq!(data["provenance"].as_array().unwrap().len(), 7);
}

fn build_dashboard_inputs(dir: &Path) -> (PathBuf, PathBuf, PathBuf) {
    let snapshot = dir.join("dash.bin");
    let events = dir.join("events.eg1");
    let report = dir.join("report.json");
    fs::write(
        &events,
        [
            r#"{"grammar":"EG-1"}"#,
            r#"{"id":1,"session_id":"dash","source":"user","verb":"assert","args":{"concept":"cpt_1","axes":[{"axis":"truth_assert","value":1.0}]}}"#,
            r#"{"id":2,"session_id":"dash","source":"user","verb":"assert","args":{"concept":"cpt_1","axes":[{"axis":"truth_assert","value":-1.0}]}}"#,
            r#"{"id":3,"session_id":"dash","source":"user","verb":"assert","args":{"concept":"cpt_1","axes":[{"axis":"truth_assert","value":1.0}]}}"#,
            r#"{"id":4,"session_id":"dash","source":"user","verb":"assert","args":{"concept":"cpt_1","axes":[{"axis":"truth_assert","value":-1.0}]}}"#,
            r#"{"id":5,"session_id":"dash","source":"user","verb":"assert","args":{"concept":"cpt_2","axes":[{"axis":"project_relevance","value":1.0},{"axis":"implementation_relevance","value":0.8},{"axis":"confidence_proxy","value":0.7}]}}"#,
            r#"{"id":6,"session_id":"dash","source":"user","verb":"link","args":{"a":"cpt_1","b":"cpt_2","weight":1.0}}"#,
            r#"{"id":7,"session_id":"dash","source":"user","verb":"goal_push","args":{"concept":"cpt_2"}}"#,
            r#"{"id":8,"session_id":"dash","source":"llm_claim","verb":"assert","args":{"concept":"cpt_3","axes":[{"axis":"confidence_proxy","value":1.0}]}}"#,
            "",
        ]
        .join("\n"),
    )
    .unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_am"))
        .args([
            "apply",
            "--snapshot",
            snapshot.to_str().unwrap(),
            "--events",
            events.to_str().unwrap(),
            "--report",
            report.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(output.status.success(), "{}", stderr(&output));
    let trace = trace_path_for(&snapshot);
    assert!(trace.exists());
    (snapshot, trace, report)
}

fn run_dashboard(snapshot: &Path, trace: &Path, report: &Path, out: &Path) {
    let output = Command::new(env!("CARGO_BIN_EXE_am"))
        .args([
            "dashboard",
            "--snapshot",
            snapshot.to_str().unwrap(),
            "--beval",
            "tests/golden/b05_beval.json",
            "--trace",
            trace.to_str().unwrap(),
            "--report",
            report.to_str().unwrap(),
            "--out",
            out.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(output.status.success(), "{}", stderr(&output));
}

fn embedded_payload(html: &str) -> Value {
    let start = html
        .find("<script id=\"am-data\" type=\"application/json\">")
        .unwrap()
        + "<script id=\"am-data\" type=\"application/json\">".len();
    let end = html[start..].find("</script>").unwrap() + start;
    serde_json::from_str(&html[start..end]).unwrap()
}

fn stderr(output: &std::process::Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

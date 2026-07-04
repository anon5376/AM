use am001::core::state::AmState;
use am001::core::step::step_result;
use am001::core::theta::Theta;
use am001::parser::rule::parse_rule_line;
use am001::storage::snapshot_file::save_snapshot;
use serde_json::Value;
use std::fs;
use std::process::Command;
use tempfile::tempdir;

const GOLDEN: &str = include_str!("golden/b05_dashboard.html");

#[test]
fn dashboard_cli_matches_golden_and_is_self_contained() {
    let dir = tempdir().unwrap();
    let snapshot = dir.path().join("dash.bin");
    let out1 = dir.path().join("dash1.html");
    let out2 = dir.path().join("dash2.html");
    save_snapshot(&snapshot, &dashboard_state()).unwrap();

    run_dashboard(&snapshot, &out1);
    run_dashboard(&snapshot, &out2);
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
}

#[test]
fn dashboard_embedded_json_smoke() {
    let data = embedded_payload(GOLDEN);
    assert_eq!(data["schema"], "AM-DASHBOARD-1");
    assert_eq!(data["snapshot"]["row_count"], 2);
    assert_eq!(
        data["snapshot"]["contradictions"].as_array().unwrap().len(),
        1
    );
    assert_eq!(data["beval"].as_array().unwrap().len(), 1);
    assert_eq!(data["beval"][0]["lane"], "b2");
}

fn run_dashboard(snapshot: &std::path::Path, out: &std::path::Path) {
    let output = Command::new(env!("CARGO_BIN_EXE_am"))
        .args([
            "dashboard",
            "--snapshot",
            snapshot.to_str().unwrap(),
            "--beval",
            "tests/golden/b05_beval.json",
            "--out",
            out.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(output.status.success(), "{}", stderr(&output));
}

fn dashboard_state() -> AmState {
    let mut state = AmState::new(Theta::default()).unwrap();
    for (event_id, line) in [
        "assert cpt_1 truth_assert=1",
        "assert cpt_1 truth_assert=-1",
        "assert cpt_1 truth_assert=1",
        "assert cpt_1 truth_assert=-1",
        "assert cpt_2 project_relevance=1 implementation_relevance=0.8 confidence_proxy=0.7",
        "link cpt_1 cpt_2 1",
        "goal push cpt_2",
    ]
    .into_iter()
    .enumerate()
    {
        let event = parse_rule_line(line, event_id as i64 + 1).unwrap();
        step_result(&mut state, &event).unwrap();
    }
    state
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

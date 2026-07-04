use am001::beval::compile::{compile_context, CONTRADICTION_DIRECTIVE, DEFAULT_CONTEXT_BUDGET};
use am001::beval::prompt::token_count;
use am001::beval::{run_beval, BevalConfig, Lane, TransportKind};
use am001::core::state::AmState;
use am001::core::step::step_result;
use am001::core::theta::Theta;
use am001::parser::rule::parse_rule_line;
use am001::storage::snapshot_file::{load_snapshot, save_snapshot};
use std::fs;
use std::process::Command;
use tempfile::tempdir;

const GOLDEN_1200: &str = include_str!("golden/b03_context_1200.txt");
const GOLDEN_400: &str = include_str!("golden/b03_context_400.txt");

#[test]
fn compile_context_golden_byte_identity() {
    let dir = tempdir().unwrap();
    let snapshot = dir.path().join("b03.bin");
    let state = b03_state();
    save_snapshot(&snapshot, &state).unwrap();
    let loaded = load_snapshot(&snapshot).unwrap();

    let first = compile_context(&loaded, DEFAULT_CONTEXT_BUDGET).unwrap();
    let second = compile_context(&loaded, DEFAULT_CONTEXT_BUDGET).unwrap();
    assert_eq!(first, second);
    assert_eq!(first, GOLDEN_1200);

    let compact = compile_context(&loaded, 400).unwrap();
    assert_eq!(compact, GOLDEN_400);
    assert!(token_count(&compact) <= 400);
}

#[test]
fn compile_context_budget_and_truncation_order() {
    let context = compile_context(&large_state_without_contradictions(), 120).unwrap();
    assert!(token_count(&context) <= 120, "{context}");
    assert!(context.contains("STATE:\n"));
    assert!(context.contains("CONTRADICTIONS:\n(none)\n"));
    assert!(!context.contains("write x"));
    assert!(!context.contains("cert 0.98\nattention cert"));
}

#[test]
fn contradiction_directive_presence_and_absence() {
    let with_contradictions = compile_context(&b03_state(), DEFAULT_CONTEXT_BUDGET).unwrap();
    assert!(with_contradictions.contains(CONTRADICTION_DIRECTIVE));

    let without_contradictions = compile_context(
        &large_state_without_contradictions(),
        DEFAULT_CONTEXT_BUDGET,
    )
    .unwrap();
    assert!(!without_contradictions.contains(CONTRADICTION_DIRECTIVE));
}

#[test]
fn compile_context_cli_writes_budgeted_output() {
    let dir = tempdir().unwrap();
    let snapshot = dir.path().join("b03.bin");
    let out = dir.path().join("context.txt");
    save_snapshot(&snapshot, &b03_state()).unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_am"))
        .args([
            "compile-context",
            "--snapshot",
            snapshot.to_str().unwrap(),
            "--budget",
            "400",
            "--out",
            out.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("compile-context out="));
    let context = fs::read_to_string(out).unwrap();
    assert_eq!(context, GOLDEN_400);
    assert!(token_count(&context) <= 400);
}

#[test]
fn b2_lane_replay_scores_synthetic_fixtures() {
    let dir = tempdir().unwrap();
    let snapshot = dir.path().join("b03.bin");
    let out = dir.path().join("b2.json");
    save_snapshot(&snapshot, &b03_state()).unwrap();

    let results = run_beval(&config(Lane::B2, &out, Some(snapshot))).unwrap();
    assert_eq!(results.evaluated, 29);
    assert_eq!(results.skipped, 0);
    assert!(results.token_cost.total_context_tokens > 0);
    let contradiction = results
        .category_scores
        .iter()
        .find(|score| score.category.to_string() == "contradiction_handling")
        .unwrap();
    assert_eq!(contradiction.matched, 5);
    assert_eq!(contradiction.total, 5);
}

#[test]
fn full_beval_replay_double_run_byte_identical_b0_b1_b2() {
    let dir = tempdir().unwrap();
    let snapshot = dir.path().join("b03.bin");
    save_snapshot(&snapshot, &b03_state()).unwrap();

    for lane in [Lane::B0, Lane::B1, Lane::B2] {
        let out1 = dir.path().join(format!("{}-1.json", lane.as_str()));
        let out2 = dir.path().join(format!("{}-2.json", lane.as_str()));
        let snapshot1 = (lane == Lane::B2).then(|| snapshot.clone());
        let snapshot2 = (lane == Lane::B2).then(|| snapshot.clone());
        let config1 = config(lane, &out1, snapshot1);
        let config2 = config(lane, &out2, snapshot2);
        run_beval(&config1).unwrap();
        run_beval(&config2).unwrap();
        assert_eq!(fs::read(&out1).unwrap(), fs::read(&out2).unwrap());
    }
}

fn b03_state() -> AmState {
    let mut state = AmState::new(Theta::default()).unwrap();
    let specs = [
        ("cpt_1", "stability"),
        ("cpt_2", "architecture_relevance"),
        ("cpt_3", "completion"),
        ("cpt_4", "persistence"),
        ("cpt_5", "reasoning_relevance"),
    ];
    let mut event_id = 1;
    for (label, axis) in specs {
        for value in [1.0_f32, -1.0, 1.0, -1.0] {
            apply_line(
                &mut state,
                event_id,
                &format!("assert {label} {axis}={value}"),
            );
            event_id += 1;
        }
    }

    for line in [
        "assert cpt_6 project_relevance=1 implementation_relevance=0.9 confidence_proxy=0.7",
        "assert cpt_7 tool_relevance=1 planning_relevance=0.8 context_relevance=0.6",
        "assert cpt_8 learning_relevance=1 specificity=0.75 attention=0.65",
        "link cpt_1 cpt_6 1.0",
        "link cpt_2 cpt_7 0.9",
        "link cpt_3 cpt_8 0.8",
        "goal push cpt_6",
    ] {
        apply_line(&mut state, event_id, line);
        event_id += 1;
    }
    for idx in 9..=28 {
        apply_line(
            &mut state,
            event_id,
            &format!(
                "assert cpt_{} project_relevance=1 implementation_relevance=0.8 attention=0.6 specificity=0.5",
                idx
            ),
        );
        event_id += 1;
    }
    for idx in 9..=20 {
        apply_line(
            &mut state,
            event_id,
            &format!("link cpt_{} cpt_{} 0.6", idx, idx + 1),
        );
        event_id += 1;
    }
    state
}

fn large_state_without_contradictions() -> AmState {
    let mut state = AmState::new(Theta::default()).unwrap();
    let mut event_id = 1;
    for idx in 1..=24 {
        apply_line(
            &mut state,
            event_id,
            &format!(
                "assert cpt_{} project_relevance=1 implementation_relevance=0.8 attention=0.6",
                idx
            ),
        );
        event_id += 1;
    }
    for idx in 1..=12 {
        apply_line(
            &mut state,
            event_id,
            &format!("link cpt_{} cpt_{} 0.7", idx, idx + 1),
        );
        event_id += 1;
    }
    state
}

fn apply_line(state: &mut AmState, event_id: i64, line: &str) {
    let event = parse_rule_line(line, event_id).unwrap();
    step_result(state, &event).unwrap();
}

fn config(lane: Lane, out: &std::path::Path, snapshot: Option<std::path::PathBuf>) -> BevalConfig {
    BevalConfig {
        corpus_dir: "beval/corpus".into(),
        lane,
        transport: TransportKind::Replay,
        fixtures_dir: "beval/fixtures/synthetic_v1".into(),
        record: false,
        out: out.to_path_buf(),
        snapshot,
        context_budget: DEFAULT_CONTEXT_BUDGET,
    }
}

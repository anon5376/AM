use am001::beval::corpus::{load_corpus, validate_corpus, AnswerKey, Category, Matcher, Task};
use am001::beval::prompt::{token_count, truncate_recency_window};
use am001::beval::scoring::score_response;
use am001::beval::transport::{prompt_hash, LlmTransport, ReplayTransport};
use am001::beval::{run_beval, BevalConfig, Lane, TransportKind};
use std::fs;
use std::process::Command;
use tempfile::tempdir;

#[test]
fn corpus_schema_validates() {
    let tasks = load_corpus(std::path::Path::new("beval/corpus")).unwrap();
    validate_corpus(&tasks).unwrap();
    assert_eq!(tasks.len(), 29);
    assert_eq!(
        tasks
            .iter()
            .filter(|task| task.category == Category::StageAccuracy)
            .count(),
        8
    );
    assert_eq!(
        tasks
            .iter()
            .filter(|task| task.category == Category::StaleClaim)
            .count(),
        5
    );
    assert_eq!(
        tasks
            .iter()
            .filter(|task| task.category == Category::ContradictionHandling)
            .count(),
        5
    );
}

#[test]
fn scorer_matchers_deterministic() {
    let exact = task_with_key(vec![Matcher::Exact {
        value: "ok".to_string(),
    }]);
    assert!(score_response(&exact, "ANSWER: ok").matched);
    assert!(!score_response(&exact, "ANSWER: nope").matched);
    assert_eq!(
        score_response(&exact, "ok").reasons,
        vec!["NoAnswerEnvelope".to_string()]
    );

    let regex = task_with_key(vec![Matcher::Regex {
        pattern: "^run-\\d+$".to_string(),
    }]);
    assert!(score_response(&regex, "ANSWER: run-42").matched);
    assert!(!score_response(&regex, "ANSWER: run-x").matched);

    let contains = task_with_key(vec![
        Matcher::MustContainAll {
            values: vec!["alpha".to_string(), "omega".to_string()],
        },
        Matcher::MustContainNone {
            values: vec!["stale".to_string()],
        },
    ]);
    assert!(score_response(&contains, "ANSWER: alpha then omega").matched);
    assert!(!score_response(&contains, "ANSWER: alpha stale omega").matched);
}

#[test]
fn replay_double_run_byte_identical() {
    for lane in [Lane::B0, Lane::B1] {
        let dir = tempdir().unwrap();
        let out1 = dir.path().join(format!("{}-1.json", lane.as_str()));
        let out2 = dir.path().join(format!("{}-2.json", lane.as_str()));
        let config1 = config(lane, &out1);
        let config2 = config(lane, &out2);
        run_beval(&config1).unwrap();
        run_beval(&config2).unwrap();
        assert_eq!(fs::read(&out1).unwrap(), fs::read(&out2).unwrap());
    }
}

#[test]
fn b2_lane_unavailable_skips_without_zero_fill() {
    let dir = tempdir().unwrap();
    let out = dir.path().join("b2.json");
    let results = run_beval(&config(Lane::B2, &out)).unwrap();
    assert_eq!(results.evaluated, 0);
    assert_eq!(results.skipped, 29);
    assert!(results
        .task_results
        .iter()
        .all(|task| task.skip_reason.as_deref() == Some("LaneUnavailable")));
}

#[test]
fn token_truncation_deterministic_and_budgeted() {
    assert_eq!(token_count(""), 0);
    assert_eq!(token_count("abcde"), 2);
    let text = format!("{}{}", "a".repeat(40), "tail");
    let first = truncate_recency_window(&text, 2);
    let second = truncate_recency_window(&text, 2);
    assert_eq!(first, second);
    assert_eq!(first.tokens, 2);
    assert_eq!(first.text, "aaaatail");
}

#[test]
fn stale_trap_scoring_correct() {
    let mut task = task_with_key(vec![
        Matcher::MustContainAll {
            values: vec!["stale".to_string(), "t=20".to_string()],
        },
        Matcher::MustContainNone {
            values: vec!["only passes at t=2".to_string()],
        },
    ]);
    task.answer_key.stale_markers = vec!["only passes at t=2".to_string()];
    let parrots = score_response(&task, "ANSWER: completion only passes at t=2");
    assert!(parrots.stale_repeated);
    assert!(!parrots.matched);

    let clean = score_response(&task, "ANSWER: stale; current default passes at t=20");
    assert!(!clean.stale_repeated);
    assert!(clean.matched);
}

#[test]
fn missing_fixture_errors_cleanly() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("manifest.json"),
        r#"{"version":"synthetic_v1","synthetic":true,"fixture_format":"sha256-prompt-v1"}"#,
    )
    .unwrap();
    let mut replay = ReplayTransport::new(dir.path()).unwrap();
    let err = replay.complete("missing prompt").unwrap_err().to_string();
    assert!(err.contains(&prompt_hash("missing prompt")));
    assert!(err.contains("prompt_preview=missing prompt"));
}

#[test]
fn live_transport_never_used_by_tests() {
    let forbidden_import = "ollama_".to_string() + "client";
    let forbidden_enable = "AM_ENABLE_".to_string() + "OLLAMA=1";
    for entry in fs::read_dir("tests").unwrap() {
        let path = entry.unwrap().path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("rs") {
            continue;
        }
        let text = fs::read_to_string(&path).unwrap();
        for line in text.lines() {
            let trimmed = line.trim_start();
            assert!(
                !(trimmed.starts_with("use ") && trimmed.contains(&forbidden_import)),
                "test source imports live transport: {}",
                path.display()
            );
            assert!(
                !trimmed.contains(&forbidden_enable),
                "test source enables live transport: {}",
                path.display()
            );
        }
    }
}

#[test]
fn apply_rejection_exit_prints_one_clean_error_line() {
    let dir = tempdir().unwrap();
    let snapshot = dir.path().join("am.bin");
    let events = dir.path().join("events.jsonl");
    fs::write(
        &events,
        r#"{"grammar":"EG-1"}
{"id":1,"session_id":"s","source":"llm_claim","verb":"cue","args":{"concept":"rust","strength":0.8}}
"#,
    )
    .unwrap();
    assert_clean_apply_rejection(&snapshot, &events, None);
    assert_clean_apply_rejection(&snapshot, &events, Some("1"));
}

fn assert_clean_apply_rejection(
    snapshot: &std::path::Path,
    events: &std::path::Path,
    rust_backtrace: Option<&str>,
) {
    let mut command = Command::new(env!("CARGO_BIN_EXE_am"));
    command.args([
        "apply",
        "--snapshot",
        snapshot.to_str().unwrap(),
        "--events",
        events.to_str().unwrap(),
    ]);
    match rust_backtrace {
        Some(value) => {
            command.env("RUST_BACKTRACE", value);
        }
        None => {
            command.env_remove("RUST_BACKTRACE");
        }
    }
    let output = command.output().unwrap();
    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    let lines = stderr.lines().collect::<Vec<_>>();
    assert_eq!(lines.len(), 1, "{stderr}");
    assert_eq!(lines[0], "Error: EG-1 apply completed with rejected events");
    assert!(!stderr.to_lowercase().contains("backtrace"));
}

fn config(lane: Lane, out: &std::path::Path) -> BevalConfig {
    BevalConfig {
        corpus_dir: "beval/corpus".into(),
        lane,
        transport: TransportKind::Replay,
        fixtures_dir: "beval/fixtures/synthetic_v1".into(),
        record: false,
        out: out.to_path_buf(),
    }
}

fn task_with_key(matchers: Vec<Matcher>) -> Task {
    Task {
        id: "task".to_string(),
        category: Category::StageAccuracy,
        question: "question".to_string(),
        answer_key: AnswerKey {
            matchers,
            stale_markers: Vec::new(),
        },
        source_ref: "source".to_string(),
        requires_lane: None,
        drift_group: None,
    }
}

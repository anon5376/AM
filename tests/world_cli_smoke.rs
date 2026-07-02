use std::fs;
use std::process::Command;
use tempfile::tempdir;

#[test]
fn world_run_writes_jsonl_and_deterministic_dump_lines() {
    let dir = tempdir().unwrap();
    let script = dir.path().join("actions.txt");
    let obs = dir.path().join("obs.jsonl");
    let trace = dir.path().join("trace.jsonl");
    fs::write(&script, "N E E PickUp S Open W Drop Wait N\n").unwrap();

    let bin = env!("CARGO_BIN_EXE_am");
    let output = Command::new(bin)
        .args([
            "world-run",
            "--map-seed",
            "7",
            "--rule-seed",
            "3",
            "--script",
        ])
        .arg(&script)
        .args(["--obs-out"])
        .arg(&obs)
        .args(["--trace-out"])
        .arg(&trace)
        .args(["--dump-every", "5"])
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("world tick=5"));
    assert!(stdout.contains("world tick=10"));
    assert!(stdout.contains("world-run actions=10"));
    assert!(fs::metadata(obs).unwrap().len() > 0);
    assert!(fs::metadata(trace).unwrap().len() > 0);
}

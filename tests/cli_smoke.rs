use std::process::Command;
use tempfile::tempdir;

#[test]
fn cli_init_step_text_and_dump_smoke() {
    let dir = tempdir().unwrap();
    let snapshot = dir.path().join("am001.bin");
    let bin = env!("CARGO_BIN_EXE_am");

    let status = Command::new(bin)
        .args(["init", "--snapshot"])
        .arg(&snapshot)
        .status()
        .unwrap();
    assert!(status.success());

    let status = Command::new(bin)
        .args(["step-text", "--snapshot"])
        .arg(&snapshot)
        .arg("assert rust truth_assert=1 goal_relevance=0.8")
        .status()
        .unwrap();
    assert!(status.success());

    let output = Command::new(bin)
        .args(["dump", "--snapshot"])
        .arg(&snapshot)
        .args(["--sort", "act", "--top", "5"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("rust"));
}

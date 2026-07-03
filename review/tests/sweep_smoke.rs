use std::fs;
use std::process::Command;
use tempfile::tempdir;

#[test]
fn tiny_sweep_writes_well_formed_csv() {
    let dir = tempdir().unwrap();
    let grid = dir.path().join("grid.json");
    let out = dir.path().join("results.csv");
    fs::write(
        &grid,
        r#"{"lam_settle":[0.55],"beta":[1.4],"gamma":[1.2],"th0":[3.0],"k_i":[3.0],"eta_w":[0.3],"del_w":[0.0],"t":20,"measure_margin":false,"scale":{"n":512,"completion_assemblies":12}}"#,
    )
    .unwrap();

    let status = Command::new(env!("CARGO_BIN_EXE_am"))
        .args(["sweep", "--grid"])
        .arg(&grid)
        .args(["--out"])
        .arg(&out)
        .status()
        .unwrap();
    assert!(status.success());

    let csv = fs::read_to_string(out).unwrap();
    let lines = csv.lines().collect::<Vec<_>>();
    assert_eq!(lines.len(), 2);
    assert!(lines[0].starts_with("theta_hash,n,completion_assemblies,valid"));
    assert_eq!(csv_columns(lines[1]), csv_columns(lines[0]));
    let headers = lines[0].split(',').collect::<Vec<_>>();
    let values = lines[1].split(',').collect::<Vec<_>>();
    let margin_idx = headers
        .iter()
        .position(|header| *header == "recall_margin_095")
        .unwrap();
    assert_eq!(values[margin_idx], "");
}

fn csv_columns(line: &str) -> usize {
    let mut in_quotes = false;
    let mut count = 1;
    for ch in line.chars() {
        match ch {
            '"' => in_quotes = !in_quotes,
            ',' if !in_quotes => count += 1,
            _ => {}
        }
    }
    count
}

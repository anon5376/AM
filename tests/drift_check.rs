use std::fs;
use std::path::Path;

#[test]
fn architecture_drift_scan_rejects_forbidden_dependencies_and_python() {
    let cargo = fs::read_to_string("Cargo.toml").unwrap();
    for forbidden in ["rusqlite", "sqlx", "duckdb", "tantivy", "rand"] {
        assert!(
            !cargo.contains(forbidden),
            "forbidden dependency found: {forbidden}"
        );
    }
    assert_no_python(Path::new("."));
    let core_sources = collect_rs(Path::new("src/core"));
    for path in core_sources {
        let text = fs::read_to_string(&path).unwrap();
        assert!(
            !text.contains("ollama"),
            "core references ollama in {}",
            path.display()
        );
        assert!(
            !text.contains("trace_file"),
            "core reads trace file in {}",
            path.display()
        );
    }
}

fn assert_no_python(path: &Path) {
    for entry in fs::read_dir(path).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.components().any(|part| part.as_os_str() == ".git") {
            continue;
        }
        if path.is_dir() {
            assert_no_python(&path);
        } else {
            assert_ne!(path.extension().and_then(|ext| ext.to_str()), Some("py"));
        }
    }
}

fn collect_rs(path: &Path) -> Vec<std::path::PathBuf> {
    let mut out = Vec::new();
    for entry in fs::read_dir(path).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_dir() {
            out.extend(collect_rs(&path));
        } else if path.extension().and_then(|ext| ext.to_str()) == Some("rs") {
            out.push(path);
        }
    }
    out
}

use std::fs;
use std::path::Path;

#[test]
fn architecture_drift_scan_rejects_forbidden_dependencies_and_python() {
    let cargo = fs::read_to_string("Cargo.toml").unwrap().to_lowercase();
    for forbidden in [
        "rusqlite", "sqlx", "duckdb", "tantivy", "qdrant", "chroma", "faiss", "hnsw", "redis",
        "postgres", "mysql", "sqlite", "rag", "rand",
    ] {
        assert!(
            !contains_token(&cargo, forbidden),
            "forbidden dependency found: {forbidden}"
        );
    }
    assert_no_python(Path::new("."));

    let core_sources = collect_rs(Path::new("src/core"));
    for path in &core_sources {
        let text = fs::read_to_string(path).unwrap();
        let lower = text.to_lowercase();
        for forbidden in [
            "ollama",
            "reqwest",
            "ureq",
            "hyper",
            "http",
            "client",
            "trace_file",
            "tokio",
            "rayon",
            "spawn",
            "systemtime",
            "utc::now",
            "local::now",
            "rand",
        ] {
            assert!(
                !lower.contains(forbidden),
                "forbidden core runtime token `{forbidden}` in {}",
                path.display()
            );
        }
    }

    for path in [
        "src/core/settle.rs",
        "src/core/write.rs",
        "src/core/hebb.rs",
        "src/core/decay.rs",
        "src/core/inspect.rs",
    ] {
        let text = fs::read_to_string(path).unwrap();
        assert!(
            !text.contains("HashMap"),
            "order-sensitive HashMap mention in {path}"
        );
    }

    for path in [
        "src/core/settle.rs",
        "src/core/write.rs",
        "src/core/hebb.rs",
        "src/core/decay.rs",
    ] {
        let text = fs::read_to_string(path).unwrap();
        for forbidden in ["labels", "label_to_id", "concept_label"] {
            assert!(
                !contains_token(&text, forbidden),
                "dynamics depend on label metadata `{forbidden}` in {path}"
            );
        }
    }

    for path in collect_rs(Path::new("src")) {
        let text = fs::read_to_string(&path).unwrap().to_lowercase();
        let in_world = path.components().any(|part| part.as_os_str() == "world");
        let in_forbidden_runtime_path = path.components().any(|part| {
            matches!(
                part.as_os_str().to_str(),
                Some("core" | "percept" | "learner" | "planner")
            )
        });
        let forbidden_words: &[&str] = if in_world {
            &[
                "key", "door", "food", "poison", "goal", "want", "opens", "unlocks",
            ]
        } else if in_forbidden_runtime_path {
            &[
                "key",
                "door",
                "food",
                "poison",
                "want",
                "opens",
                "unlocks",
                "portable",
                "barrier",
                "consumable",
                "hazard",
                "exit",
                "wall",
            ]
        } else {
            &[]
        };
        for forbidden in forbidden_words {
            assert!(
                !contains_token(&text, forbidden),
                "semantic leak `{forbidden}` in {}",
                path.display()
            );
        }
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
    out.sort();
    out
}

fn contains_token(text: &str, needle: &str) -> bool {
    text.split(|ch: char| !ch.is_ascii_alphanumeric() && ch != '_')
        .any(|token| token == needle)
}

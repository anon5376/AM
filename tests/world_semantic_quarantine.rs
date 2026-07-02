use am001::world::runner::run_episode;
use am001::world::script::parse_script;
use am001::world::theta::WorldTheta;
use std::fs;
use std::path::{Path, PathBuf};

#[test]
fn world_observations_and_runtime_paths_stay_quarantined() {
    let actions = parse_script("N E PickUp Open Wait").unwrap();
    let output = run_episode(WorldTheta::default(), 7, 3, &actions, 0).unwrap();
    assert_no_semantic_tokens("world observation jsonl", &output.observation_jsonl);

    for path in existing_roots(["src/core", "src/percept", "src/learner", "src/planner"]) {
        for file in collect_rs(&path) {
            let text = fs::read(&file).unwrap();
            assert_no_semantic_tokens(&file.display().to_string(), &text);
        }
    }

    let cargo = fs::read_to_string("Cargo.toml").unwrap().to_lowercase();
    for forbidden in [
        "rusqlite", "sqlx", "duckdb", "tantivy", "qdrant", "chroma", "faiss", "hnsw", "redis",
        "postgres", "mysql", "sqlite", "rag", "rand",
    ] {
        assert!(
            !contains_token(&cargo, forbidden),
            "forbidden dep `{forbidden}`"
        );
    }
    assert_no_python(Path::new("."));

    for root in existing_roots(["src/core", "src/world"]) {
        for file in collect_rs(&root) {
            let text = fs::read_to_string(&file).unwrap().to_lowercase();
            for forbidden in [
                "ollama",
                "reqwest",
                "ureq",
                "hyper",
                "http",
                "client",
                "tokio",
                "rayon",
                "spawn",
                "systemtime",
                "utc::now",
                "local::now",
                "rand",
            ] {
                assert!(
                    !text.contains(forbidden),
                    "forbidden runtime token `{forbidden}` in {}",
                    file.display()
                );
            }
        }
    }
}

fn existing_roots<const N: usize>(roots: [&str; N]) -> Vec<PathBuf> {
    roots
        .iter()
        .map(PathBuf::from)
        .filter(|path| path.exists())
        .collect()
}

fn collect_rs(path: &Path) -> Vec<PathBuf> {
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

fn assert_no_semantic_tokens(name: &str, bytes: &[u8]) {
    let text = String::from_utf8_lossy(bytes).to_lowercase();
    for word in [
        "key", "door", "food", "hazard", "poison", "opens", "unlocks",
    ] {
        assert!(
            !contains_token(&text, word),
            "semantic leak `{word}` in {name}"
        );
    }
}

fn contains_token(text: &str, needle: &str) -> bool {
    text.split(|ch: char| !ch.is_ascii_alphanumeric() && ch != '_')
        .any(|token| token == needle)
}

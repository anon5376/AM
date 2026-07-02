use am001::world::runner::run_episode;
use am001::world::script::parse_script;
use am001::world::theta::WorldTheta;
use sha2::{Digest, Sha256};

#[test]
fn same_seeds_and_script_produce_byte_identical_jsonl() {
    let actions = parse_script("N E E PickUp S Open W Drop Wait N").unwrap();
    let theta = WorldTheta::default();
    let left = run_episode(theta.clone(), 7, 3, &actions, 3).unwrap();
    let right = run_episode(theta.clone(), 7, 3, &actions, 3).unwrap();
    assert_eq!(left.observation_jsonl, right.observation_jsonl);
    assert_eq!(left.trace_jsonl, right.trace_jsonl);
    assert_eq!(left.dumps, right.dumps);

    let different = run_episode(theta, 8, 3, &actions, 3).unwrap();
    assert_ne!(
        hash(&left.observation_jsonl),
        hash(&different.observation_jsonl)
    );
    assert_ne!(hash(&left.trace_jsonl), hash(&different.trace_jsonl));
}

fn hash(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

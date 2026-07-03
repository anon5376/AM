use am001::world::runner::run_episode;
use am001::world::script::parse_script;
use am001::world::theta::WorldTheta;
use sha2::{Digest, Sha256};

#[test]
fn fixed_script_matches_golden_world_hashes() {
    let actions = parse_script("N E E PickUp S Open W Drop Wait N").unwrap();
    let output = run_episode(WorldTheta::default(), 7, 3, &actions, 10).unwrap();
    let obs_hash = hash(&output.observation_jsonl);
    let trace_hash = hash(&output.trace_jsonl);
    println!("world_golden_obs_hash={obs_hash}");
    println!("world_golden_trace_hash={trace_hash}");
    assert_eq!(
        obs_hash,
        "6998ec720a46695ef15521e2b0effe24e7da817f8d3de0ba459682af4b0d2be2"
    );
    assert_eq!(
        trace_hash,
        "83790d725e130218eaffa82b2421109aeed3200a424992119187d624b061effc"
    );
}

fn hash(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

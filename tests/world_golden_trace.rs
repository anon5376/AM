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
        "c0129c72e355a41f1b74ae2cadb07c8a82606d454c71011a640aadf417bc6229"
    );
    assert_eq!(
        trace_hash,
        "4f3b634b3ebd56231098d6fd0700a5a9e840b20a286579ff05bd9b46d162c699"
    );
}

fn hash(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

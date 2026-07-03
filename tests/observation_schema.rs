use am001::world::observation::Observation;
use am001::world::runner::run_episode;
use am001::world::script::parse_script;
use am001::world::theta::WorldTheta;
use serde_json::Value;
use std::collections::BTreeSet;

#[test]
fn observations_roundtrip_and_use_only_neutral_fields() {
    let actions = parse_script("N E PickUp Wait").unwrap();
    let output = run_episode(WorldTheta::default(), 7, 3, &actions, 0).unwrap();
    for line in output.observation_jsonl.split(|byte| *byte == b'\n') {
        if line.is_empty() {
            continue;
        }
        let observation: Observation = serde_json::from_slice(line).unwrap();
        let value: Value = serde_json::from_slice(line).unwrap();
        assert_eq!(
            object_keys(&value),
            BTreeSet::from([
                "blocked",
                "energy_bucket",
                "held_shape_id",
                "map_seed",
                "reward_delta",
                "rule_seed",
                "self",
                "tick",
                "visible_entities",
            ])
        );
        for entity in &observation.visible_entities {
            assert!(entity.local_id > 0);
            assert!(entity.shape_id > 0);
            assert!(entity.color_id > 0);
            assert!((1..=3).contains(&entity.size));
        }
        if let Some(shape) = observation.held_shape_id {
            assert!(shape > 0);
        }
        assert_no_blacklist_words(&String::from_utf8(line.to_vec()).unwrap());
    }
}

fn object_keys(value: &Value) -> BTreeSet<&str> {
    value
        .as_object()
        .unwrap()
        .keys()
        .map(|key| key.as_str())
        .collect()
}

fn assert_no_blacklist_words(text: &str) {
    let lower = text.to_lowercase();
    for word in [
        "key",
        "door",
        "food",
        "poison",
        "opens",
        "unlocks",
        "portable",
        "barrier",
        "consumable",
        "hazard",
        "exit",
        "wall",
    ] {
        assert!(
            !lower
                .split(|ch: char| !ch.is_ascii_alphanumeric() && ch != '_')
                .any(|token| token == word),
            "semantic leak `{word}` in observation json: {text}"
        );
    }
}

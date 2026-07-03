use am001::percept::PerceptBridge;
use am001::world::runner::run_episode;
use am001::world::script::parse_script;
use am001::world::theta::WorldTheta;

#[test]
fn same_observation_produces_identical_events_from_same_initial_bridge_state() {
    let actions = parse_script("N W PickUp E").unwrap();
    let output = run_episode(WorldTheta::default(), 7, 3, &actions, 0).unwrap();
    let observation = output.observations[0].clone();

    let mut left = PerceptBridge::new();
    let mut right = PerceptBridge::new();
    let left_events = left.events_for_observation(&observation).unwrap();
    let right_events = right.events_for_observation(&observation).unwrap();

    assert_eq!(left_events, right_events);
    assert_eq!(
        serde_json::to_vec(&left_events).unwrap(),
        serde_json::to_vec(&right_events).unwrap()
    );
}

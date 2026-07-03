use am001::core::axes::axis_index;
use am001::percept::PerceptBridge;
use am001::world::runner::run_episode;
use am001::world::script::parse_script;
use am001::world::theta::WorldTheta;

#[test]
fn percept_events_pass_validator_and_use_existing_axes() {
    let actions = parse_script("N W PickUp E S S W Open").unwrap();
    let output = run_episode(WorldTheta::default(), 7, 3, &actions, 0).unwrap();
    let mut bridge = PerceptBridge::new();

    for observation in &output.observations {
        for event in bridge.events_for_observation(observation).unwrap() {
            let mut validated = event.clone();
            validated.validate_and_clamp().unwrap();
            for assertion in &validated.asserts {
                for axis in assertion.targets.keys() {
                    assert!(axis_index(axis).is_some(), "unknown axis {axis}");
                }
            }
        }
    }
}

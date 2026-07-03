use am001::core::event::ConceptRef;
use am001::percept::PerceptBridge;
use am001::world::runner::run_episode;
use am001::world::script::parse_script;
use am001::world::theta::WorldTheta;

#[test]
fn percept_labels_are_opaque_and_events_do_not_leak_world_words() {
    let actions = parse_script("N W PickUp E S S W Open").unwrap();
    let output = run_episode(WorldTheta::default(), 7, 3, &actions, 0).unwrap();
    let mut bridge = PerceptBridge::new();

    for observation in &output.observations {
        let events = bridge.events_for_observation(observation).unwrap();
        let json = serde_json::to_string(&events).unwrap();
        assert_no_blacklist_words(&json);
        for event in &events {
            for cue in &event.cues {
                assert_label(&cue.concept);
            }
            for assertion in &event.asserts {
                assert_label(&assertion.concept);
            }
            for link in &event.links {
                assert_label(&link.left);
                assert_label(&link.right);
            }
        }
    }
}

fn assert_label(reference: &ConceptRef) {
    let ConceptRef::Label(label) = reference else {
        panic!("percept emitted non-label reference");
    };
    let mut parts = label.split('_');
    let prefix = parts.next().unwrap_or_default();
    let digits = parts.next().unwrap_or_default();
    assert!(parts.next().is_none(), "bad label {label}");
    assert!(matches!(prefix, "loc" | "trk" | "cpt"), "bad label {label}");
    assert!(!digits.is_empty(), "bad label {label}");
    assert!(
        digits.chars().all(|ch| ch.is_ascii_digit()),
        "bad label {label}"
    );
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
            "semantic leak `{word}` in event json: {text}"
        );
    }
}

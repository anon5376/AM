mod common;

use am001::core::snapshot::from_bytes;
use am001::core::state::AmState;
use am001::core::step::step_result;
use common::{assert_event, link_event, theta_default};

#[test]
fn snapshot_roundtrip_is_byte_exact() {
    let mut state = AmState::new(theta_default()).unwrap();
    step_result(&mut state, &assert_event(1, "am001", &[("agency", 0.8)])).unwrap();
    step_result(
        &mut state,
        &assert_event(2, "rust", &[("truth_assert", 1.0)]),
    )
    .unwrap();
    step_result(&mut state, &link_event(3, "rust", "am001", 1.0)).unwrap();
    let bytes = state.snapshot_bytes();
    let loaded = from_bytes(&bytes).unwrap();
    assert_eq!(bytes, loaded.snapshot_bytes());
    assert_eq!(state.state_hash(), loaded.state_hash());
}

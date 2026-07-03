mod common;

use am001::core::inspect::{axes_report, dump_state};
use am001::core::snapshot::from_bytes;
use am001::core::state::AmState;
use am001::core::step::step_result;
use common::{assert_event, theta_default};

#[test]
fn snapshot_version_mismatch_is_refused_clearly() {
    let state = AmState::new(theta_default()).unwrap();
    let mut bytes = state.snapshot_bytes();
    bytes[0] = 1;
    let err = from_bytes(&bytes).unwrap_err();
    assert!(err.to_string().contains("format_version"));
}

#[test]
fn snapshot_theta_violating_a15c_is_refused_clearly() {
    let mut state = AmState::new(theta_default()).unwrap();
    state.theta.th0 = 0.0;
    let bytes = state.snapshot_bytes();
    let err = from_bytes(&bytes).unwrap_err();
    assert!(err.to_string().contains("A15c resting-field invariant"));
}

#[test]
fn dump_shows_min_axis_certainty_and_axes_lists_all_axes() {
    let mut state = AmState::new(theta_default()).unwrap();
    step_result(
        &mut state,
        &assert_event(1, "rust", &[("truth_assert", 1.0)]),
    )
    .unwrap();
    step_result(
        &mut state,
        &assert_event(2, "rust", &[("truth_assert", -1.0)]),
    )
    .unwrap();
    step_result(
        &mut state,
        &assert_event(3, "rust", &[("truth_assert", 1.0)]),
    )
    .unwrap();
    step_result(
        &mut state,
        &assert_event(4, "rust", &[("truth_assert", -1.0)]),
    )
    .unwrap();

    let dump = dump_state(&state, "act", 10).unwrap();
    assert!(dump.contains("min-axis cert: truth_assert"));
    assert!(dump.contains("contradiction open: truth_assert"));

    let axes = axes_report(&state, "rust").unwrap();
    assert!(axes.contains("truth_assert"));
    assert!(axes.contains("value="));
    assert_eq!(axes.lines().count(), 49);
}

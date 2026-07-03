mod common;

use am001::core::axes::axis_index;
use am001::core::state::{AmState, ContradictionStatus};
use am001::core::step::step_result;
use common::{assert_event, axis, row, theta_default};

#[test]
fn alternating_evidence_opens_and_same_sign_evidence_closes() {
    let mut state = AmState::new(theta_default());
    let truth = axis("truth_assert");

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
    let trace = step_result(
        &mut state,
        &assert_event(4, "rust", &[("truth_assert", -1.0)]),
    )
    .unwrap();

    let rust = row(&state, "rust");
    assert!(!trace.opened_contradictions.is_empty());
    assert!(state.v_get(rust, truth) > state.theta.th_v);
    assert!(state.m_get(rust, truth).abs() < 1.0);
    assert!(state.open_contradictions.iter().any(|contradiction| {
        contradiction.concept == state.row_ref(rust)
            && contradiction.axis == truth
            && contradiction.status == ContradictionStatus::Open
    }));

    for id in 5..=24 {
        step_result(
            &mut state,
            &assert_event(id, "rust", &[("truth_assert", 1.0)]),
        )
        .unwrap();
    }
    assert!(state.open_contradictions.iter().any(|contradiction| {
        contradiction.concept == state.row_ref(rust)
            && contradiction.axis == axis_index("truth_assert").unwrap()
            && matches!(contradiction.status, ContradictionStatus::Closed { .. })
    }));
}

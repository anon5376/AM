mod common;

use am001::core::state::AmState;
use am001::core::step::step_result;
use common::{assert_event, axis, row, theta_default};

#[test]
fn repeated_fact_reinforces_one_row_without_duplicates() {
    let mut state = AmState::new(theta_default()).unwrap();
    let truth = axis("truth_assert");
    let mut last_m = f32::NEG_INFINITY;
    let mut last_b = f32::NEG_INFINITY;

    for id in 1..=10 {
        step_result(
            &mut state,
            &assert_event(
                id,
                "rust",
                &[("truth_assert", 1.0), ("goal_relevance", 0.8)],
            ),
        )
        .unwrap();
        let rust = row(&state, "rust");
        let current_m = state.m_get(rust, truth);
        let current_b = state.b[rust];
        assert!(current_m + state.theta.eps_log >= last_m);
        assert!(current_b + state.theta.eps_log >= last_b);
        last_m = current_m;
        last_b = current_b;
    }

    let rust_rows = state
        .labels
        .iter()
        .filter(|label| label.as_deref() == Some("rust"))
        .count();
    assert_eq!(rust_rows, 1);
    assert!((state.m_get(row(&state, "rust"), truth) - 1.0).abs() < 0.001);
}

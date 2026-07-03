mod common;

use am001::core::state::AmState;
use am001::core::step::step_result;
use am001::core::theta::Theta;
use am001::core::{event::Event, hebb::link_map};
use common::assert_event;

#[test]
fn empty_ticks_drive_allocated_field_quiet_under_a15c() {
    let theta = Theta {
        eta_w: 0.0,
        ..Theta::default()
    };
    let mut state = AmState::new(theta).unwrap();

    for idx in 0..40 {
        step_result(
            &mut state,
            &assert_event(
                idx + 1,
                &format!("rest{idx}"),
                &[("truth_assert", if idx % 2 == 0 { 0.3 } else { -0.3 })],
            ),
        )
        .unwrap();
    }

    for row in state.allocated_rows_sorted() {
        state.a[row] = 0.0;
        state.b[row] = 0.001 * (row % 11) as f32;
        state.links[row].clear();
    }
    assert!(link_map(&state).is_empty());

    for id in 100..130 {
        let trace = step_result(&mut state, &Event::empty(id)).unwrap();
        assert!(
            trace.active_set_before_decay.is_empty(),
            "empty tick {id} had active rows {:?}",
            trace.active_set_before_decay
        );
    }

    let max_resting = state
        .allocated_rows_sorted()
        .into_iter()
        .map(|row| state.a[row])
        .fold(0.0_f32, f32::max);
    assert!(
        max_resting < state.theta.th_act,
        "max resting activation {max_resting} >= th_act {}",
        state.theta.th_act
    );
}

mod common;

use am001::core::event::{ConceptRef, Cue, Event};
use am001::core::state::AmState;
use am001::core::step::step_result;
use am001::core::trace::MutationTarget;
use common::{assert_event, row, theta_default};

#[test]
fn reused_row_rejects_stale_rowref_and_generation_mutations_are_logged() {
    let mut state = AmState::new(theta_default());
    step_result(
        &mut state,
        &assert_event(1, "old", &[("truth_assert", 0.4)]),
    )
    .unwrap();
    let old_row = row(&state, "old");
    let old_ref = state.row_ref(old_row);

    state.b[old_row] = 0.0;
    state.a[old_row] = 0.0;
    state.force_last_touched(old_row, state.tick - state.theta.a_old - 1);
    let free_trace = step_result(&mut state, &Event::empty(2)).unwrap();
    assert!(!state.is_allocated(old_row));
    assert!(free_trace.mutations.iter().any(|mutation| {
        mutation.target == MutationTarget::Generation && mutation.row == old_row
    }));

    let alloc_trace = step_result(
        &mut state,
        &assert_event(3, "new", &[("truth_assert", 0.7)]),
    )
    .unwrap();
    let new_row = row(&state, "new");
    assert_eq!(new_row, old_row);
    assert_ne!(state.row_ref(new_row), old_ref);
    assert!(alloc_trace.mutations.iter().any(|mutation| {
        mutation.target == MutationTarget::Generation && mutation.row == new_row
    }));

    let tick_before = state.tick;
    let snapshot_before = state.snapshot_bytes();
    let stale_event = Event {
        id: 4,
        cues: vec![Cue {
            concept: ConceptRef::Id(old_ref),
            strength: 0.8,
        }],
        asserts: Vec::new(),
        links: Vec::new(),
        goal_ops: Vec::new(),
    };
    let err = step_result(&mut state, &stale_event).unwrap_err();
    assert!(err.to_string().contains("stale row ref"));
    assert_eq!(state.tick, tick_before);
    assert_eq!(state.snapshot_bytes(), snapshot_before);
    println!(
        "stale_rowref_demo: stale_ref={}:{} current_ref={}:{} tick_unchanged={}",
        old_ref.id,
        old_ref.gen,
        state.row_ref(new_row).id,
        state.row_ref(new_row).gen,
        state.tick
    );
}

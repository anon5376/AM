mod common;

use am001::core::event::{ConceptRef, Event, GoalOp};
use am001::core::state::{AmState, ContradictionStatus, OpenContradiction};
use am001::core::step::step_result;
use common::{assert_event, axis, row, theta_default};

#[test]
fn stale_low_baseline_rows_free_merge_and_protected_rows_survive() {
    let mut state = AmState::new(theta_default()).unwrap();
    step_result(
        &mut state,
        &assert_event(1, "old", &[("truth_assert", 0.4)]),
    )
    .unwrap();
    let old = row(&state, "old");
    state.b[old] = 0.0;
    state.a[old] = 0.0;
    state.force_last_touched(old, state.tick - state.theta.a_old - 1);
    step_result(&mut state, &Event::empty(2)).unwrap();
    assert!(!state.is_allocated(old));

    let mut merge_state = AmState::new(theta_default()).unwrap();
    step_result(
        &mut merge_state,
        &assert_event(1, "m1", &[("truth_assert", 1.0)]),
    )
    .unwrap();
    step_result(
        &mut merge_state,
        &assert_event(2, "m2", &[("truth_assert", 0.99)]),
    )
    .unwrap();
    let m1 = row(&merge_state, "m1");
    let m2 = row(&merge_state, "m2");
    let m1_ref = merge_state.row_ref(m1);
    let m2_ref = merge_state.row_ref(m2);
    merge_state.b[m1] = 0.0;
    merge_state.b[m2] = 0.0;
    merge_state.a[m1] = 0.0;
    merge_state.a[m2] = 0.0;
    merge_state.force_last_touched(m1, merge_state.tick - merge_state.theta.a_old - 1);
    merge_state.force_last_touched(m2, merge_state.tick - merge_state.theta.a_old - 1);
    step_result(&mut merge_state, &Event::empty(3)).unwrap();
    assert_eq!(merge_state.aliases.get(&m1_ref), Some(&m2_ref));
    assert!(merge_state.is_allocated(m2));

    let mut protected = AmState::new(theta_default()).unwrap();
    step_result(
        &mut protected,
        &assert_event(1, "goalish", &[("goal_relevance", 1.0)]),
    )
    .unwrap();
    let goalish = row(&protected, "goalish");
    let goal_event = Event {
        id: 2,
        cues: Vec::new(),
        asserts: Vec::new(),
        links: Vec::new(),
        goal_ops: vec![GoalOp::Push(ConceptRef::Label("goalish".to_string()))],
    };
    step_result(&mut protected, &goal_event).unwrap();
    protected.b[goalish] = 0.0;
    protected.a[goalish] = 0.0;
    protected.force_last_touched(goalish, protected.tick - protected.theta.a_old - 1);
    step_result(&mut protected, &Event::empty(3)).unwrap();
    assert!(protected.is_allocated(goalish));

    let truth = axis("truth_assert");
    protected.open_contradictions.push(OpenContradiction {
        concept: protected.row_ref(goalish),
        axis: truth,
        evidence_event_ids: vec![1, 2, 3, 4],
        opened_tick: protected.tick,
        pressure: protected.theta.rho_c,
        status: ContradictionStatus::Open,
    });
    protected.goals.clear();
    protected.force_last_touched(goalish, protected.tick - protected.theta.a_old - 1);
    step_result(&mut protected, &Event::empty(4)).unwrap();
    assert!(protected.is_allocated(goalish));
}

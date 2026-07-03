mod common;

use am001::core::event::Event;
use am001::core::hebb::link_map;
use am001::core::snapshot::from_bytes;
use am001::core::state::AmState;
use am001::core::step::step_result;
use am001::core::trace::MutationTarget;
use common::{assert_event, cue_event, link_event, row, theta_default};

#[test]
fn freed_row_reuse_cannot_inherit_stale_w_edges() {
    let mut theta = theta_default();
    theta.del_w = 0.0;
    theta.th_merge = 2.0;
    let mut state = AmState::new(theta).unwrap();

    step_result(&mut state, &assert_event(1, "a", &[("truth_assert", 0.8)])).unwrap();
    step_result(&mut state, &assert_event(2, "b", &[("agency", 0.8)])).unwrap();
    step_result(&mut state, &link_event(3, "a", "b", 1.0)).unwrap();

    let a = row(&state, "a");
    let b = row(&state, "b");
    assert!(link_map(&state).contains_key(&(a, b)));
    assert!(link_map(&state).contains_key(&(b, a)));

    state.b[b] = 0.0;
    state.a[b] = 0.0;
    state.force_last_touched(b, state.tick - state.theta.a_old - 1);
    let free_trace = step_result(&mut state, &Event::empty(4)).unwrap();

    let removed_w_records = free_trace
        .mutations
        .iter()
        .filter(|mutation| {
            mutation.target == MutationTarget::W
                && mutation.after == 0.0
                && (mutation.row == b || mutation.target_row == Some(b))
        })
        .count();
    assert!(removed_w_records >= 2);
    assert!(!state.is_allocated(b));
    assert!(state.links[b].is_empty());
    assert!(state
        .links
        .iter()
        .all(|links| links.iter().all(|link| link.target != b)));

    for value in &mut state.a {
        *value = 0.0;
    }
    state.theta.eta_w = 0.0;
    step_result(
        &mut state,
        &assert_event(5, "c", &[("goal_relevance", 0.8)]),
    )
    .unwrap();
    let c = row(&state, "c");
    assert_eq!(c, b);
    assert!(state
        .links
        .iter()
        .all(|links| links.iter().all(|link| link.target != c)));

    state.a[c] = 0.0;
    state.theta.beta = 0.0;
    let cue_trace = step_result(&mut state, &cue_event(6, "a", 0.9)).unwrap();
    assert!(!cue_trace.active_set_before_decay.contains(&c));
    assert!(state.a[c] < state.theta.th_act);
    assert!(state
        .links
        .iter()
        .all(|links| links.iter().all(|link| link.target != c)));

    let bytes = state.snapshot_bytes();
    let loaded = from_bytes(&bytes).unwrap();
    assert_eq!(bytes, loaded.snapshot_bytes());

    println!(
        "stale_w_links_demo: freed_row={b} reused_row={c} removed_w_records={removed_w_records} c_active_after_cue={:.6}",
        state.a[c]
    );
}

mod common;

use am001::core::event::Event;
use am001::core::hebb::link_map;
use am001::core::state::AmState;
use am001::core::step::step_result;
use am001::core::theta::Theta;
use am001::core::trace::{Cause, MutationTarget};
use common::{assert_event, link_event, row};

#[test]
fn nonzero_link_decay_decreases_and_prunes_with_cause_records() {
    let mut theta = Theta::default();
    theta.del_w = 0.01;
    theta.eps_w = 0.1;
    theta.eta_w = 0.15;
    println!("theta_hash={}", theta.hash());
    let mut state = AmState::new(theta).unwrap();

    step_result(
        &mut state,
        &assert_event(1, "left", &[("truth_assert", 1.0)]),
    )
    .unwrap();
    step_result(&mut state, &assert_event(2, "right", &[("agency", 1.0)])).unwrap();
    step_result(&mut state, &link_event(3, "left", "right", 1.0)).unwrap();
    let left = row(&state, "left");
    let right = row(&state, "right");
    let initial = link_map(&state)[&(left, right)];
    assert!(initial > state.theta.eps_w);

    let mut saw_decay = false;
    let mut saw_prune = false;
    let mut last_seen_weight = initial;
    for id in 4..54 {
        let trace = step_result(&mut state, &Event::empty(id)).unwrap();
        for mutation in &trace.mutations {
            if mutation.target == MutationTarget::W
                && mutation.row == left
                && mutation.target_row == Some(right)
            {
                if mutation.cause == Cause::LinkDecay {
                    saw_decay = true;
                    assert!(mutation.after < mutation.before);
                    last_seen_weight = mutation.after;
                }
                if mutation.cause == Cause::LinkPrune {
                    saw_prune = true;
                    assert_eq!(mutation.after, 0.0);
                }
            }
        }
    }

    assert!(saw_decay, "expected at least one LinkDecay record");
    assert!(saw_prune, "expected eps_w LinkPrune record");
    assert!(last_seen_weight < initial);
    assert!(!link_map(&state).contains_key(&(left, right)));
    assert!(!link_map(&state).contains_key(&(right, left)));
}

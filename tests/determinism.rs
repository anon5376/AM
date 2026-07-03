mod common;

use am001::core::diff::trace_hash;
use am001::core::snapshot::from_bytes;
use am001::core::state::AmState;
use am001::core::step::step_result;
use common::{assert_event, link_event, theta_default};

#[test]
fn fixed_stream_and_midway_snapshot_are_deterministic() {
    let theta = theta_default();
    let events = vec![
        assert_event(1, "am001", &[("goal_relevance", 1.0), ("agency", 0.8)]),
        assert_event(2, "rust", &[("truth_assert", 1.0), ("goal_relevance", 0.8)]),
        link_event(3, "rust", "am001", 1.0),
        assert_event(4, "rust", &[("truth_assert", -1.0)]),
        assert_event(5, "rust", &[("truth_assert", 1.0)]),
    ];

    let mut left = AmState::new(theta.clone()).unwrap();
    let mut left_traces = Vec::new();
    for event in &events {
        left_traces.push(step_result(&mut left, event).unwrap());
    }

    let mut right = AmState::new(theta.clone()).unwrap();
    let mut right_traces = Vec::new();
    for event in &events {
        right_traces.push(step_result(&mut right, event).unwrap());
    }

    assert_eq!(left.state_hash(), right.state_hash());
    assert_eq!(
        trace_hash(&left_traces).unwrap(),
        trace_hash(&right_traces).unwrap()
    );

    let mut interrupted = AmState::new(theta).unwrap();
    let mut interrupted_traces = Vec::new();
    for event in &events[..3] {
        interrupted_traces.push(step_result(&mut interrupted, event).unwrap());
    }
    let bytes = interrupted.snapshot_bytes();
    let mut continued = from_bytes(&bytes).unwrap();
    assert_eq!(bytes, continued.snapshot_bytes());
    for event in &events[3..] {
        interrupted_traces.push(step_result(&mut continued, event).unwrap());
    }

    assert_eq!(left.state_hash(), continued.state_hash());
    assert_eq!(
        trace_hash(&left_traces).unwrap(),
        trace_hash(&interrupted_traces).unwrap()
    );
}

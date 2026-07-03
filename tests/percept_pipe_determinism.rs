use am001::core::diff::trace_hash;
use am001::core::state::AmState;
use am001::core::step::step_result;
use am001::core::theta::Theta;
use am001::percept::PerceptBridge;
use am001::world::runner::run_episode;
use am001::world::script::parse_script;
use am001::world::theta::WorldTheta;

#[test]
fn world_percept_core_double_run_is_byte_deterministic() {
    let left = run_pipe_once();
    let right = run_pipe_once();
    assert_eq!(left.state_hash, right.state_hash);
    assert_eq!(left.trace_hash, right.trace_hash);
    assert_eq!(left.trace_bytes, right.trace_bytes);
}

fn run_pipe_once() -> PipeRun {
    let script = std::iter::repeat_n("Wait", 50)
        .collect::<Vec<_>>()
        .join(" ");
    let actions = parse_script(&script).unwrap();
    let episode = run_episode(WorldTheta::default(), 7, 3, &actions, 0).unwrap();
    assert_eq!(episode.observations.len(), 50);

    let mut bridge = PerceptBridge::new();
    let mut state = AmState::new(Theta::default()).unwrap();
    let mut traces = Vec::new();
    for observation in &episode.observations {
        let events = bridge.events_for_observation(observation).unwrap();
        for event in events {
            traces.push(step_result(&mut state, &event).unwrap());
        }
    }
    let trace_bytes = am001::core::diff::trace_jsonl_bytes(&traces).unwrap();
    PipeRun {
        state_hash: state.state_hash(),
        trace_hash: trace_hash(&traces).unwrap(),
        trace_bytes,
    }
}

struct PipeRun {
    state_hash: String,
    trace_hash: String,
    trace_bytes: Vec<u8>,
}

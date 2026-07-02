#![allow(dead_code)]

use am001::core::axes::axis_index;
use am001::core::event::{Assert, ConceptRef, Cue, Event, LinkAssert};
use am001::core::state::AmState;
use am001::core::theta::Theta;
use std::collections::BTreeMap;
use std::path::Path;

pub fn theta_default() -> Theta {
    let theta = Theta::default();
    println!("theta_hash={}", theta.hash());
    theta
}

pub fn theta_file(name: &str) -> Theta {
    let path = Path::new("tests")
        .join("theta")
        .join(format!("{name}.json"));
    let theta = Theta::from_path(path).expect("load theta file");
    println!("theta_hash={}", theta.hash());
    theta
}

pub fn assert_event(id: i64, label: &str, targets: &[(&str, f32)]) -> Event {
    let mut map = BTreeMap::new();
    for (axis, value) in targets {
        map.insert((*axis).to_string(), *value);
    }
    Event {
        id,
        cues: Vec::new(),
        asserts: vec![Assert {
            concept: ConceptRef::Label(label.to_string()),
            targets: map,
            weight: 1.0,
        }],
        links: Vec::new(),
        goal_ops: Vec::new(),
    }
}

pub fn cue_event(id: i64, label: &str, strength: f32) -> Event {
    Event {
        id,
        cues: vec![Cue {
            concept: ConceptRef::Label(label.to_string()),
            strength,
        }],
        asserts: Vec::new(),
        links: Vec::new(),
        goal_ops: Vec::new(),
    }
}

pub fn cue_two_event(id: i64, left: &str, right: &str, strength: f32) -> Event {
    Event {
        id,
        cues: vec![
            Cue {
                concept: ConceptRef::Label(left.to_string()),
                strength,
            },
            Cue {
                concept: ConceptRef::Label(right.to_string()),
                strength,
            },
        ],
        asserts: Vec::new(),
        links: Vec::new(),
        goal_ops: Vec::new(),
    }
}

pub fn link_event(id: i64, left: &str, right: &str, hint: f32) -> Event {
    Event {
        id,
        cues: Vec::new(),
        asserts: Vec::new(),
        links: vec![LinkAssert {
            left: ConceptRef::Label(left.to_string()),
            right: ConceptRef::Label(right.to_string()),
            hint,
        }],
        goal_ops: Vec::new(),
    }
}

pub fn row(state: &AmState, label: &str) -> usize {
    state
        .resolve_existing(&ConceptRef::Label(label.to_string()))
        .unwrap()
}

pub fn axis(name: &str) -> usize {
    axis_index(name).unwrap()
}

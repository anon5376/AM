mod common;

use am001::core::event::Event;
use am001::core::state::AmState;
use am001::core::step::step_result;
use common::{assert_event, cue_two_event, link_event, row, theta_default};
use std::collections::BTreeSet;

#[test]
fn partial_cue_completes_linked_assemblies_without_foreign_spill() {
    let theta = theta_default();
    let mut state = AmState::new(theta);
    let mut event_id = 1;
    let mut assemblies = Vec::new();

    for assembly in 0..30 {
        let labels = (0..5)
            .map(|member| format!("asm{assembly}_{member}"))
            .collect::<Vec<_>>();
        for label in &labels {
            let signature = if assembly % 2 == 0 { 1.0 } else { -1.0 };
            let axis_value = ((assembly as f32 % 7.0) - 3.0) / 3.0;
            step_result(
                &mut state,
                &assert_event(
                    event_id,
                    label,
                    &[
                        ("memory_relevance", signature),
                        ("project_relevance", axis_value),
                        ("truth_assert", 0.5),
                    ],
                ),
            )
            .unwrap();
            event_id += 1;
        }
        for left in 0..5 {
            for right in (left + 1)..5 {
                for _ in 0..2 {
                    step_result(
                        &mut state,
                        &link_event(event_id, &labels[left], &labels[right], 1.0),
                    )
                    .unwrap();
                    event_id += 1;
                }
            }
        }
        state.a.fill(0.0);
        assemblies.push(labels);
    }

    let trained = state.clone();
    let mut recall_sum = 0.0;
    let mut foreign_sum = 0.0;

    for labels in &assemblies {
        let mut trial = trained.clone();
        step_result(&mut trial, &Event::empty(event_id)).unwrap();
        event_id += 1;
        let trace = step_result(
            &mut trial,
            &cue_two_event(event_id, &labels[0], &labels[1], 0.8),
        )
        .unwrap();
        event_id += 1;
        let active = trace
            .active_set_before_decay
            .iter()
            .copied()
            .collect::<BTreeSet<_>>();
        let assembly_rows = labels
            .iter()
            .map(|label| row(&trial, label))
            .collect::<BTreeSet<_>>();
        let non_cued = labels[2..]
            .iter()
            .filter(|label| active.contains(&row(&trial, label)))
            .count();
        recall_sum += non_cued as f32 / 3.0;
        let foreign = active
            .iter()
            .filter(|row_id| !assembly_rows.contains(row_id))
            .count();
        foreign_sum += if active.is_empty() {
            0.0
        } else {
            foreign as f32 / active.len() as f32
        };
    }

    let mean_recall = recall_sum / assemblies.len() as f32;
    let mean_foreign = foreign_sum / assemblies.len() as f32;
    assert!(mean_recall >= 0.90, "mean recall {mean_recall}");
    assert!(mean_foreign < 0.05, "mean foreign {mean_foreign}");
}

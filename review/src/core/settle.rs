use crate::core::state::AmState;
use crate::core::trace::{Cause, StepTrace};

#[derive(Clone, Debug)]
pub struct SettleResult {
    pub active_set: Vec<usize>,
    pub iterations: usize,
    pub max_delta: f32,
}

pub fn settle(state: &mut AmState, trace: &mut StepTrace) -> SettleResult {
    let rows = state.allocated_rows_sorted();
    if rows.is_empty() {
        return SettleResult {
            active_set: Vec::new(),
            iterations: 0,
            max_delta: 0.0,
        };
    }

    let mut current = state.a.clone();
    let before = state.a.clone();
    let mut iterations = 0;
    let mut max_delta = 0.0;

    for iter in 0..state.theta.t {
        iterations = iter + 1;
        let mean_a = rows.iter().map(|row| current[*row]).sum::<f32>() / rows.len() as f32;
        let mut next = current.clone();
        max_delta = 0.0;

        for row in &rows {
            let mut link_sum = 0.0;
            for link in &state.links[*row] {
                link_sum += link.weight * current[link.target];
            }

            let mut goal_pull = 0.0;
            for goal_ref in &state.goals {
                let Ok(goal_ref) = state.resolve_row_ref(*goal_ref) else {
                    continue;
                };
                let goal = goal_ref.id;
                let mut sq = 0.0;
                for axis in 0..state.theta.d {
                    let diff = state.m_get(*row, axis) - state.m_get(goal, axis);
                    sq += diff * diff;
                }
                goal_pull +=
                    state.b[goal] * (-(sq / state.theta.d as f32) / state.theta.tau_g).exp();
            }

            let net = link_sum + state.b[*row] + goal_pull - state.theta.gamma * mean_a;
            let sigmoid = 1.0 / (1.0 + (-(net - state.theta.th0)).exp());
            let value = (state.theta.lam_settle * current[*row] + state.theta.beta * sigmoid)
                .clamp(0.0, 1.0);
            let delta = (value - current[*row]).abs();
            if delta > max_delta {
                max_delta = delta;
            }
            next[*row] = value;
        }

        current = next;
        if max_delta < state.theta.eps {
            break;
        }
    }

    let mut ranked: Vec<(usize, f32)> = rows.iter().map(|row| (*row, current[*row])).collect();
    ranked.sort_by(|left, right| {
        right
            .1
            .total_cmp(&left.1)
            .then_with(|| left.0.cmp(&right.0))
    });
    let keep: Vec<usize> = ranked
        .iter()
        .filter(|(_, value)| *value >= state.theta.th_act)
        .take(state.theta.a_max)
        .map(|(row, _)| *row)
        .collect();

    for row in &rows {
        if !keep.contains(row) {
            current[*row] = 0.0;
        }
    }

    for row in &rows {
        state.a_set_logged(*row, current[*row], trace, Cause::Settle);
    }

    let active_set = state.active_rows_sorted(state.theta.th_act);
    let _ = before;
    SettleResult {
        active_set,
        iterations,
        max_delta,
    }
}

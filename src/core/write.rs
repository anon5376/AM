use crate::core::axes::axis_index;
use crate::core::event::Event;
use crate::core::state::AmState;
use crate::core::trace::{Cause, StepTrace};
use anyhow::{Context, Result};
use std::collections::BTreeSet;

pub fn write_asserts(
    state: &mut AmState,
    event: &Event,
    trace: &mut StepTrace,
    written_cells: &mut BTreeSet<(usize, usize)>,
) -> Result<()> {
    for assert in &event.asserts {
        let row = state.resolve_existing(&assert.concept)?;
        if state.a[row] < state.theta.th_write {
            continue;
        }
        for (axis_name, target) in &assert.targets {
            let axis =
                axis_index(axis_name).with_context(|| format!("unknown axis {axis_name}"))?;
            let before_m = state.m_get(row, axis);
            let evidence_delta = *target - before_m;
            let coordinate_delta =
                state.theta.eta_m * assert.weight * state.a[row] * evidence_delta;
            if coordinate_delta.abs() >= state.theta.eps_log {
                state.m_set_logged(row, axis, before_m + coordinate_delta, trace, Cause::Write);
            }

            let after_m = state.m_get(row, axis);
            let residual = *target - after_m;
            let before_v = state.v_get(row, axis);
            let after_v =
                state.theta.lam_v * before_v + (1.0 - state.theta.lam_v) * residual * residual;
            state.v_set_logged(row, axis, after_v, trace, Cause::VarianceUpdate);

            if evidence_delta.abs() >= state.theta.eps_log {
                let sign = if evidence_delta > 0.0 { 1 } else { -1 };
                state.push_write_evidence(row, axis, event.id, sign, coordinate_delta);
                written_cells.insert((row, axis));
            }
        }
    }
    Ok(())
}

use crate::world::action::Action;
use crate::world::grid::{GridWorld, TerminationCause, WorldRenderFrame, WorldTrace};
use crate::world::observation::Observation;
use crate::world::theta::WorldTheta;
use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

#[derive(Clone, Debug)]
pub struct EpisodeOutput {
    pub observations: Vec<Observation>,
    pub traces: Vec<WorldTrace>,
    pub observation_jsonl: Vec<u8>,
    pub trace_jsonl: Vec<u8>,
    pub dumps: Vec<String>,
    pub render_frames: Vec<WorldRenderFrame>,
    pub termination: TerminationCause,
}

pub fn run_episode(
    theta: WorldTheta,
    map_seed: u64,
    rule_seed: u64,
    actions: &[Action],
    dump_every: usize,
) -> Result<EpisodeOutput> {
    let mut world = GridWorld::new(theta, map_seed, rule_seed)?;
    let mut observations = Vec::new();
    let mut traces = Vec::new();
    let mut dumps = Vec::new();
    let mut render_frames = Vec::new();
    let mut termination = None;

    for action in actions {
        let (observation, trace) = world.step(*action);
        if dump_every > 0 && (observation.tick as usize).is_multiple_of(dump_every) {
            dumps.push(world.dump_line());
        }
        render_frames.push(world.render_frame());
        termination = trace.termination;
        observations.push(observation);
        traces.push(trace);
        if termination.is_some() {
            break;
        }
    }

    let termination = termination.unwrap_or(TerminationCause::ScriptEnd);
    if let Some(trace) = traces.last_mut() {
        if trace.termination.is_none() {
            trace.termination = Some(termination);
        }
    }
    if let Some(frame) = render_frames.last_mut() {
        if frame.termination.is_none() {
            frame.termination = Some(termination);
        }
    }

    let observation_jsonl = jsonl_bytes(&observations)?;
    let trace_jsonl = jsonl_bytes(&traces)?;
    Ok(EpisodeOutput {
        observations,
        traces,
        observation_jsonl,
        trace_jsonl,
        dumps,
        render_frames,
        termination,
    })
}

pub fn write_episode_files(output: &EpisodeOutput, obs_out: &Path, trace_out: &Path) -> Result<()> {
    write_bytes(obs_out, &output.observation_jsonl)?;
    write_bytes(trace_out, &output.trace_jsonl)?;
    Ok(())
}

fn write_bytes(path: &Path, bytes: &[u8]) -> Result<()> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)
                .with_context(|| format!("create directory {}", parent.display()))?;
        }
    }
    fs::write(path, bytes).with_context(|| format!("write {}", path.display()))
}

fn jsonl_bytes<T: serde::Serialize>(items: &[T]) -> Result<Vec<u8>> {
    let mut out = Vec::new();
    for item in items {
        out.extend(serde_json::to_vec(item).context("serialize world jsonl item")?);
        out.push(b'\n');
    }
    Ok(out)
}

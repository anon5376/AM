use crate::apply::{
    apply_parsed_batch, header_rejection_report, load_staging, parse_batch_text, save_staging,
    staging_path_for, trace_path_for, write_report, write_trace_file, ApplyReport,
};
use crate::cli::render::{format_frame_with_glyphs, EpisodeGlyphs};
use crate::cli::repl::run_repl;
use crate::core::event::Event;
use crate::core::inspect::{axes_report, dump_state, format_diff};
use crate::core::state::AmState;
use crate::core::step::step_result;
use crate::core::theta::Theta;
use crate::eval::sweep::{load_grid, run_sweep};
use crate::parser::rule::parse_rule_line;
use crate::percept::PerceptBridge;
use crate::storage::snapshot_file::{load_snapshot, save_snapshot};
use crate::storage::trace_file::append_trace;
use crate::world::runner::{run_episode, write_episode_files};
use crate::world::script::parse_script_path;
use crate::world::theta::WorldTheta;
use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::time::Instant;

#[derive(Parser)]
#[command(name = "am")]
#[command(about = "AM001 parameter-memory attractor engine")]
pub struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    Init {
        #[arg(long)]
        snapshot: PathBuf,
        #[arg(long)]
        theta: Option<PathBuf>,
    },
    Dump {
        #[arg(long)]
        snapshot: PathBuf,
        #[arg(long)]
        theta: Option<PathBuf>,
        #[arg(long, default_value = "act")]
        sort: String,
        #[arg(long, default_value_t = 50)]
        top: usize,
    },
    Axes {
        #[arg(long)]
        snapshot: PathBuf,
        label: String,
        #[arg(long)]
        theta: Option<PathBuf>,
    },
    Step {
        #[arg(long)]
        snapshot: PathBuf,
        #[arg(long)]
        event: PathBuf,
        #[arg(long)]
        theta: Option<PathBuf>,
        #[arg(long)]
        diff: bool,
    },
    StepText {
        #[arg(long)]
        snapshot: PathBuf,
        text: String,
        #[arg(long)]
        theta: Option<PathBuf>,
        #[arg(long)]
        diff: bool,
    },
    Apply {
        #[arg(long)]
        snapshot: PathBuf,
        #[arg(long)]
        events: PathBuf,
        #[arg(long)]
        report: Option<PathBuf>,
    },
    Run {
        #[arg(long)]
        snapshot: PathBuf,
        #[arg(long)]
        events: PathBuf,
        #[arg(long)]
        trace: Option<PathBuf>,
        #[arg(long)]
        theta: Option<PathBuf>,
    },
    Repl {
        #[arg(long)]
        snapshot: PathBuf,
        #[arg(long)]
        theta: Option<PathBuf>,
    },
    BenchStep {
        #[arg(long)]
        snapshot: PathBuf,
        #[arg(long, default_value_t = 1000)]
        events: usize,
        #[arg(long)]
        theta: Option<PathBuf>,
    },
    Sweep {
        #[arg(long)]
        grid: PathBuf,
        #[arg(long)]
        out: PathBuf,
    },
    WorldRun {
        #[arg(long)]
        map_seed: u64,
        #[arg(long)]
        rule_seed: u64,
        #[arg(long)]
        script: PathBuf,
        #[arg(long)]
        obs_out: PathBuf,
        #[arg(long)]
        trace_out: PathBuf,
        #[arg(long, default_value_t = 0)]
        dump_every: usize,
        #[arg(long)]
        render: bool,
        #[arg(long)]
        theta: Option<PathBuf>,
    },
    Pipe {
        #[arg(long)]
        map_seed: u64,
        #[arg(long)]
        rule_seed: u64,
        #[arg(long)]
        script: PathBuf,
        #[arg(long, default_value_t = 0)]
        dump_every: usize,
        #[arg(long)]
        theta: Option<PathBuf>,
        #[arg(long)]
        world_theta: Option<PathBuf>,
    },
}

pub fn run() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Init { snapshot, theta } => {
            let state = AmState::new(Theta::load_optional(theta.as_deref())?)?;
            save_snapshot(snapshot, &state)?;
        }
        Command::Dump {
            snapshot,
            theta,
            sort,
            top,
        } => {
            let mut state = load_snapshot(snapshot)?;
            apply_theta_override(&mut state, theta.as_deref())?;
            print!("{}", dump_state(&state, &sort, top)?);
        }
        Command::Axes {
            snapshot,
            label,
            theta,
        } => {
            let mut state = load_snapshot(snapshot)?;
            apply_theta_override(&mut state, theta.as_deref())?;
            print!("{}", axes_report(&state, &label)?);
        }
        Command::Step {
            snapshot,
            event,
            theta,
            diff,
        } => {
            let mut state = load_or_new(&snapshot, theta.as_deref())?;
            apply_theta_override(&mut state, theta.as_deref())?;
            let mut event = read_event_json(&event)?;
            event.validate_and_clamp()?;
            let trace = step_result(&mut state, &event)?;
            if diff {
                print!("{}", format_diff(&state, &trace));
            }
            save_snapshot(snapshot, &state)?;
        }
        Command::StepText {
            snapshot,
            text,
            theta,
            diff,
        } => {
            let mut state = load_or_new(&snapshot, theta.as_deref())?;
            apply_theta_override(&mut state, theta.as_deref())?;
            let event = parse_rule_line(&text, state.tick + 1)?;
            let trace = step_result(&mut state, &event)?;
            if diff {
                print!("{}", format_diff(&state, &trace));
            }
            save_snapshot(snapshot, &state)?;
        }
        Command::Apply {
            snapshot,
            events,
            report,
        } => {
            let text = fs::read_to_string(&events)
                .with_context(|| format!("read EG-1 events {}", events.display()))?;
            let parsed = match parse_batch_text(&text) {
                Ok(parsed) => parsed,
                Err(rejection) => {
                    let report_doc = header_rejection_report(rejection);
                    emit_apply_report(report.as_deref(), &report_doc)?;
                    anyhow::bail!("EG-1 file rejected before apply");
                }
            };
            if parsed.has_structural_rejection() {
                let report_doc = ApplyReport::from_verdicts(
                    parsed.rejections.clone(),
                    parsed.structural_rejections,
                );
                emit_apply_report(report.as_deref(), &report_doc)?;
                anyhow::bail!("EG-1 file has structural rejection");
            }

            let staging_path = staging_path_for(&snapshot);
            let trace_path = trace_path_for(&snapshot);
            let mut state = load_or_new(&snapshot, None)?;
            let mut staging = load_staging(&staging_path)?;
            let mut outcome = apply_parsed_batch(&mut state, &parsed, &mut staging)?;
            outcome.report.staging_file = Some(staging_path.display().to_string());
            outcome.report.trace_file = Some(trace_path.display().to_string());
            save_snapshot(&snapshot, &state)?;
            save_staging(&staging_path, &staging)?;
            write_trace_file(&trace_path, &outcome.traces)?;
            emit_apply_report(report.as_deref(), &outcome.report)?;
            if outcome.report.has_rejections() {
                anyhow::bail!("EG-1 apply completed with rejected events");
            }
        }
        Command::Run {
            snapshot,
            events,
            trace,
            theta,
        } => {
            let mut state = load_or_new(&snapshot, theta.as_deref())?;
            apply_theta_override(&mut state, theta.as_deref())?;
            let file =
                fs::File::open(&events).with_context(|| format!("open {}", events.display()))?;
            for line in BufReader::new(file).lines() {
                let line = line?;
                if line.trim().is_empty() {
                    continue;
                }
                let mut event: Event = serde_json::from_str(&line).context("parse event jsonl")?;
                event.validate_and_clamp()?;
                let step_trace = step_result(&mut state, &event)?;
                if let Some(path) = &trace {
                    append_trace(path, &step_trace)?;
                }
            }
            save_snapshot(snapshot, &state)?;
        }
        Command::Repl { snapshot, theta } => {
            let mut state = load_or_new(&snapshot, theta.as_deref())?;
            apply_theta_override(&mut state, theta.as_deref())?;
            run_repl(&mut state)?;
            save_snapshot(snapshot, &state)?;
        }
        Command::BenchStep {
            snapshot,
            events,
            theta,
        } => {
            let mut state = load_or_new(&snapshot, theta.as_deref())?;
            apply_theta_override(&mut state, theta.as_deref())?;
            let mut max = std::time::Duration::ZERO;
            let mut total = std::time::Duration::ZERO;
            for idx in 0..events {
                let event = Event::empty(state.tick + 1 + idx as i64);
                let started = Instant::now();
                let _ = step_result(&mut state, &event)?;
                let elapsed = started.elapsed();
                total += elapsed;
                max = max.max(elapsed);
            }
            let mean = if events == 0 {
                std::time::Duration::ZERO
            } else {
                total / events as u32
            };
            println!(
                "bench-step events={} mean_us={} max_us={}",
                events,
                mean.as_micros(),
                max.as_micros()
            );
            save_snapshot(snapshot, &state)?;
        }
        Command::Sweep { grid, out } => {
            let grid = load_grid(&grid)?;
            let summary = run_sweep(&grid, &out)?;
            println!(
                "wrote {} total_points={} invalid_points={} evaluated_points={} all_pass_points={}",
                out.display(),
                summary.total_points,
                summary.invalid_points,
                summary.evaluated_points,
                summary.all_pass_points
            );
        }
        Command::WorldRun {
            map_seed,
            rule_seed,
            script,
            obs_out,
            trace_out,
            dump_every,
            render,
            theta,
        } => {
            let theta = WorldTheta::load_optional(theta.as_deref())?;
            let actions = parse_script_path(&script)?;
            let output = run_episode(theta, map_seed, rule_seed, &actions, dump_every)?;
            write_episode_files(&output, &obs_out, &trace_out)?;
            for line in output.dumps {
                println!("{line}");
            }
            if render {
                if let Some(first) = output.render_frames.first() {
                    let mut glyphs = EpisodeGlyphs::from_first_frame(first);
                    for frame in &output.render_frames {
                        print!("{}", format_frame_with_glyphs(frame, &mut glyphs));
                    }
                }
            }
            println!(
                "world-run actions={} ran={} termination={} obs={} trace={}",
                actions.len(),
                output.traces.len(),
                output.termination,
                obs_out.display(),
                trace_out.display()
            );
        }
        Command::Pipe {
            map_seed,
            rule_seed,
            script,
            dump_every,
            theta,
            world_theta,
        } => {
            let world_theta = WorldTheta::load_optional(world_theta.as_deref())?;
            let actions = parse_script_path(&script)?;
            let output = run_episode(world_theta, map_seed, rule_seed, &actions, 0)?;
            let mut bridge = PerceptBridge::new();
            let mut state = AmState::new(Theta::load_optional(theta.as_deref())?)?;
            let mut emitted = 0_usize;
            for observation in &output.observations {
                let events = bridge.events_for_observation(observation)?;
                for event in events {
                    let _ = step_result(&mut state, &event)?;
                    emitted += 1;
                }
                if dump_every > 0 && (observation.tick as usize).is_multiple_of(dump_every) {
                    println!(
                        "pipe tick={} core_tick={} emitted_events={}",
                        observation.tick, state.tick, emitted
                    );
                    print!("{}", dump_state(&state, "act", 8)?);
                }
            }
            println!(
                "pipe actions={} observations={} events={} core_tick={} termination={}",
                actions.len(),
                output.observations.len(),
                emitted,
                state.tick,
                output.termination
            );
        }
    }
    Ok(())
}

fn load_or_new(path: &Path, theta_path: Option<&Path>) -> Result<AmState> {
    if path.exists() {
        load_snapshot(path)
    } else {
        AmState::new(Theta::load_optional(theta_path)?)
    }
}

fn apply_theta_override(state: &mut AmState, theta_path: Option<&Path>) -> Result<()> {
    if let Some(path) = theta_path {
        state.theta = Theta::from_path(path)?;
    }
    Ok(())
}

fn read_event_json(path: &Path) -> Result<Event> {
    let text =
        fs::read_to_string(path).with_context(|| format!("read event {}", path.display()))?;
    serde_json::from_str(&text).context("parse event json")
}

fn emit_apply_report(path: Option<&Path>, report: &ApplyReport) -> Result<()> {
    if let Some(path) = path {
        write_report(path, report)
    } else {
        println!(
            "{}",
            serde_json::to_string_pretty(report).context("serialize apply report")?
        );
        Ok(())
    }
}

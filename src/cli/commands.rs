use crate::cli::repl::run_repl;
use crate::core::event::Event;
use crate::core::inspect::{dump_state, format_diff};
use crate::core::state::AmState;
use crate::core::step::step_result;
use crate::core::theta::Theta;
use crate::parser::rule::parse_rule_line;
use crate::storage::snapshot_file::{load_snapshot, save_snapshot};
use crate::storage::trace_file::append_trace;
use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

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
}

pub fn run() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Init { snapshot, theta } => {
            let state = AmState::new(Theta::load_optional(theta.as_deref())?);
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
            for idx in 0..events {
                let event = Event::empty(state.tick + 1 + idx as i64);
                let _ = step_result(&mut state, &event)?;
            }
            save_snapshot(snapshot, &state)?;
        }
    }
    Ok(())
}

fn load_or_new(path: &Path, theta_path: Option<&Path>) -> Result<AmState> {
    if path.exists() {
        load_snapshot(path)
    } else {
        Ok(AmState::new(Theta::load_optional(theta_path)?))
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

use crate::core::event::{Assert, ConceptRef, Cue, Event, GoalOp, LinkAssert};
use anyhow::{Context, Result};
use std::collections::BTreeMap;

pub fn parse_rule_line(input: &str, event_id: i64) -> Result<Event> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Ok(Event::empty(event_id));
    }
    let parts = trimmed.split_whitespace().collect::<Vec<_>>();
    match parts.first().copied() {
        Some("assert") | Some("remember") => parse_assert(&parts, event_id),
        Some("cue") => parse_cue(&parts, event_id),
        Some("link") => parse_link(&parts, event_id),
        Some("goal") => parse_goal(&parts, event_id),
        Some(other) => anyhow::bail!("unknown command {other}"),
        None => Ok(Event::empty(event_id)),
    }
}

fn parse_assert(parts: &[&str], event_id: i64) -> Result<Event> {
    anyhow::ensure!(parts.len() >= 3, "assert requires label and axis=value");
    let label = parts[1].to_string();
    let mut weight = 1.0;
    let mut targets = BTreeMap::new();
    let mut idx = 2;
    while idx < parts.len() {
        if parts[idx] == "--weight" {
            idx += 1;
            anyhow::ensure!(idx < parts.len(), "--weight needs a value");
            weight = parts[idx].parse::<f32>().context("parse weight")?;
        } else {
            let (axis, value) = parts[idx]
                .split_once('=')
                .with_context(|| format!("expected axis=value, got {}", parts[idx]))?;
            targets.insert(
                axis.to_string(),
                value.parse::<f32>().context("parse axis value")?,
            );
        }
        idx += 1;
    }
    anyhow::ensure!(
        !targets.is_empty(),
        "assert requires at least one target axis"
    );
    let mut event = Event {
        id: event_id,
        cues: Vec::new(),
        asserts: vec![Assert {
            concept: ConceptRef::Label(label),
            targets,
            weight,
        }],
        links: Vec::new(),
        goal_ops: Vec::new(),
    };
    event.validate_and_clamp()?;
    Ok(event)
}

fn parse_cue(parts: &[&str], event_id: i64) -> Result<Event> {
    anyhow::ensure!(
        parts.len() == 2 || parts.len() == 3,
        "cue requires label [strength]"
    );
    let strength = if parts.len() == 3 {
        parts[2].parse::<f32>().context("parse cue strength")?
    } else {
        1.0
    };
    let mut event = Event {
        id: event_id,
        cues: vec![Cue {
            concept: ConceptRef::Label(parts[1].to_string()),
            strength,
        }],
        asserts: Vec::new(),
        links: Vec::new(),
        goal_ops: Vec::new(),
    };
    event.validate_and_clamp()?;
    Ok(event)
}

fn parse_link(parts: &[&str], event_id: i64) -> Result<Event> {
    anyhow::ensure!(
        parts.len() == 3 || parts.len() == 4,
        "link requires left right [hint]"
    );
    let hint = if parts.len() == 4 {
        parts[3].parse::<f32>().context("parse link hint")?
    } else {
        1.0
    };
    let mut event = Event {
        id: event_id,
        cues: Vec::new(),
        asserts: Vec::new(),
        links: vec![LinkAssert {
            left: ConceptRef::Label(parts[1].to_string()),
            right: ConceptRef::Label(parts[2].to_string()),
            hint,
        }],
        goal_ops: Vec::new(),
    };
    event.validate_and_clamp()?;
    Ok(event)
}

fn parse_goal(parts: &[&str], event_id: i64) -> Result<Event> {
    anyhow::ensure!(parts.len() == 3, "goal requires push|pop label");
    let op = match parts[1] {
        "push" => GoalOp::Push(ConceptRef::Label(parts[2].to_string())),
        "pop" => GoalOp::Pop(ConceptRef::Label(parts[2].to_string())),
        other => anyhow::bail!("unknown goal op {other}"),
    };
    let mut event = Event {
        id: event_id,
        cues: Vec::new(),
        asserts: Vec::new(),
        links: Vec::new(),
        goal_ops: vec![op],
    };
    event.validate_and_clamp()?;
    Ok(event)
}

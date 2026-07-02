use crate::world::action::Action;
use anyhow::{Context, Result};
use std::fs;
use std::path::Path;
use std::str::FromStr;

pub fn parse_script(text: &str) -> Result<Vec<Action>> {
    let mut actions = Vec::new();
    for (line_idx, line) in text.lines().enumerate() {
        let content = line.split('#').next().unwrap_or("").trim();
        if content.is_empty() {
            continue;
        }
        for token in content.split(|ch: char| ch.is_whitespace() || ch == ',') {
            if token.is_empty() {
                continue;
            }
            let action = Action::from_str(token)
                .with_context(|| format!("parse action on line {}", line_idx + 1))?;
            actions.push(action);
        }
    }
    Ok(actions)
}

pub fn parse_script_path(path: &Path) -> Result<Vec<Action>> {
    let text =
        fs::read_to_string(path).with_context(|| format!("read script {}", path.display()))?;
    parse_script(&text)
}

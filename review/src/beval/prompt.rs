use crate::beval::corpus::Task;
use crate::beval::Lane;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct LaneContext {
    pub text: String,
    pub tokens: usize,
}

impl LaneContext {
    pub fn empty() -> Self {
        Self {
            text: String::new(),
            tokens: 0,
        }
    }
}

pub fn build_prompt(lane: Lane, task: &Task, context: &LaneContext) -> String {
    match lane {
        Lane::B0 => format!(
            "AM001 Track B eval lane=b0\nReturn exactly one line in this envelope:\nANSWER: <one line>\nNo extra text.\nTask id: {}\nCategory: {}\nQuestion:\n{}\n",
            task.id, task.category, task.question
        ),
        Lane::B1 => format!(
            "AM001 Track B eval lane=b1\nReturn exactly one line in this envelope:\nANSWER: <one line>\nNo extra text.\nRaw log context follows. It may contain stale traps; use current verified artifacts when asked for current state.\n[RAWLOG_BEGIN]\n{}\n[RAWLOG_END]\nTask id: {}\nCategory: {}\nQuestion:\n{}\n",
            context.text, task.id, task.category, task.question
        ),
        Lane::B2 => format!(
            "AM001 Track B eval lane=b2\nReturn exactly one line in this envelope:\nANSWER: <one line>\nNo extra text.\nCompiled AM context follows.\n[AM_CONTEXT_BEGIN]\n{}\n[AM_CONTEXT_END]\nTask id: {}\nCategory: {}\nQuestion:\n{}\n",
            context.text, task.id, task.category, task.question
        ),
    }
}

pub fn load_b1_context(fixtures_dir: &Path, budget_tokens: usize) -> Result<LaneContext> {
    let rawlog = rawlog_path(fixtures_dir);
    let text = fs::read_to_string(&rawlog).with_context(|| format!("read {}", rawlog.display()))?;
    Ok(truncate_recency_window(&text, budget_tokens))
}

pub fn truncate_recency_window(text: &str, budget_tokens: usize) -> LaneContext {
    let max_chars = budget_tokens.saturating_mul(4);
    let truncated = if text.len() <= max_chars {
        text.to_string()
    } else {
        let mut start = text.len() - max_chars;
        while start < text.len() && !text.is_char_boundary(start) {
            start += 1;
        }
        text[start..].to_string()
    };
    LaneContext {
        tokens: token_count(&truncated),
        text: truncated,
    }
}

pub fn token_count(text: &str) -> usize {
    text.len().div_ceil(4)
}

pub fn context_token_count(context: &LaneContext) -> usize {
    context.tokens
}

fn rawlog_path(fixtures_dir: &Path) -> std::path::PathBuf {
    fixtures_dir
        .parent()
        .map(|parent| parent.join("rawlog_v1.md"))
        .unwrap_or_else(|| Path::new("beval/fixtures/rawlog_v1.md").to_path_buf())
}

#[cfg(test)]
mod tests {
    use super::{token_count, truncate_recency_window};

    #[test]
    fn token_count_ceil_chars_over_four() {
        assert_eq!(token_count(""), 0);
        assert_eq!(token_count("a"), 1);
        assert_eq!(token_count("abcd"), 1);
        assert_eq!(token_count("abcde"), 2);
    }

    #[test]
    fn truncation_keeps_tail() {
        let ctx = truncate_recency_window("aaaabbbbcccc", 2);
        assert_eq!(ctx.text, "bbbbcccc");
        assert_eq!(ctx.tokens, 2);
    }
}

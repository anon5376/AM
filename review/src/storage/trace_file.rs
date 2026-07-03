use crate::core::trace::StepTrace;
use anyhow::{Context, Result};
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;

pub fn append_trace(path: impl AsRef<Path>, trace: &StepTrace) -> Result<()> {
    if let Some(parent) = path.as_ref().parent() {
        std::fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path.as_ref())
        .with_context(|| format!("open trace {}", path.as_ref().display()))?;
    let line = serde_json::to_vec(trace).context("serialize trace")?;
    file.write_all(&line).context("write trace line")?;
    file.write_all(b"\n").context("write trace newline")?;
    Ok(())
}

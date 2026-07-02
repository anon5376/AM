use crate::core::snapshot::{from_bytes, to_bytes};
use crate::core::state::AmState;
use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

pub fn save_snapshot(path: impl AsRef<Path>, state: &AmState) -> Result<()> {
    if let Some(parent) = path.as_ref().parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    fs::write(path.as_ref(), to_bytes(state))
        .with_context(|| format!("write snapshot {}", path.as_ref().display()))
}

pub fn load_snapshot(path: impl AsRef<Path>) -> Result<AmState> {
    let bytes = fs::read(path.as_ref())
        .with_context(|| format!("read snapshot {}", path.as_ref().display()))?;
    from_bytes(&bytes)
}

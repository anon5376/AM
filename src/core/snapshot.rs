use crate::core::state::AmState;
use anyhow::{Context, Result};

pub fn to_bytes(state: &AmState) -> Vec<u8> {
    state.snapshot_bytes()
}

pub fn from_bytes(bytes: &[u8]) -> Result<AmState> {
    bincode::deserialize(bytes).context("deserialize AM snapshot")
}

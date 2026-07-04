use crate::core::state::{AmState, SNAPSHOT_FORMAT_VERSION};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct SnapshotWire {
    format_version: u32,
    state: AmState,
}

pub fn to_bytes(state: &AmState) -> Vec<u8> {
    bincode::serialize(&SnapshotWire {
        format_version: state.format_version,
        state: state.clone(),
    })
    .expect("snapshot serialization cannot fail")
}

pub fn from_bytes(bytes: &[u8]) -> Result<AmState> {
    let wire: SnapshotWire = bincode::deserialize(bytes)
        .context("deserialize AM snapshot v5 envelope; older snapshots are not compatible")?;
    anyhow::ensure!(
        wire.format_version == SNAPSHOT_FORMAT_VERSION,
        "snapshot format_version {} is incompatible with expected {}",
        wire.format_version,
        SNAPSHOT_FORMAT_VERSION
    );
    anyhow::ensure!(
        wire.state.format_version == SNAPSHOT_FORMAT_VERSION,
        "snapshot state format_version {} is incompatible with expected {}",
        wire.state.format_version,
        SNAPSHOT_FORMAT_VERSION
    );
    wire.state.theta.validate()?;
    Ok(wire.state)
}

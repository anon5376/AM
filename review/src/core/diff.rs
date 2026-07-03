use crate::core::trace::StepTrace;
use anyhow::{Context, Result};
use sha2::{Digest, Sha256};

pub fn trace_jsonl_bytes(traces: &[StepTrace]) -> Result<Vec<u8>> {
    let mut out = Vec::new();
    for trace in traces {
        let line = serde_json::to_vec(trace).context("serialize trace json")?;
        out.extend(line);
        out.push(b'\n');
    }
    Ok(out)
}

pub fn trace_hash(traces: &[StepTrace]) -> Result<String> {
    let bytes = trace_jsonl_bytes(traces)?;
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    Ok(format!("{:x}", hasher.finalize()))
}

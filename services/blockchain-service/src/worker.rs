//! Squads v4 only has a maintained TypeScript SDK — no Rust equivalent — so
//! the actual on-chain calls happen in `contracts/squads/chain-worker.ts`,
//! invoked here as a subprocess. This module is the one place that knows
//! how to talk to it; everything else in this service just calls
//! `run_chain_worker` and gets JSON back.

use std::process::Stdio;
use tokio::process::Command;

pub async fn run_chain_worker(
    contracts_dir: &str,
    action: &str,
    args_json: &str,
) -> Result<serde_json::Value, String> {
    let output = Command::new("node")
        .arg("node_modules/ts-node/dist/bin.js")
        .arg("squads/chain-worker.ts")
        .arg(action)
        .arg(args_json)
        .env("TS_NODE_TRANSPILE_ONLY", "true")
        .current_dir(contracts_dir)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await
        .map_err(|e| format!("failed to spawn chain-worker: {e}"))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    if !output.status.success() {
        let status = output.status;
        return Err(format!(
            "chain-worker exited with {status}: stdout={stdout} stderr={stderr}"
        ));
    }

    // chain-worker prints exactly one JSON line on success. Everything else
    // on stdout (e.g. the harmless "bigint: Failed to load bindings"
    // warning some environments print) isn't JSON, so take the last
    // non-empty line rather than the whole buffer.
    let json_line = stdout
        .lines()
        .rev()
        .find(|l| !l.trim().is_empty())
        .ok_or_else(|| format!("no output from chain-worker: stderr={stderr}"))?;

    serde_json::from_str(json_line)
        .map_err(|e| format!("failed to parse chain-worker output '{json_line}': {e}"))
}

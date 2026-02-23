// SPDX-License-Identifier: Apache-2.0

use std::process::Command;

use super::types::CollectError;

/// Run zsh's alias command and return the raw output.
pub fn collect_raw_aliases() -> Result<String, CollectError> {
    let output = Command::new("zsh")
        .args(["-ic", "alias"])
        .output()
        .map_err(|e| CollectError::SpawnFailed { source: e })?;

    if !output.status.success() && output.stdout.is_empty() {
        return Err(CollectError::ShellFailed {
            status: output.status,
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        });
    }

    String::from_utf8(output.stdout).map_err(|e| CollectError::InvalidUtf8 { source: e })
}

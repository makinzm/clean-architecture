use anyhow::Result;
use std::process::Command;

/// Returns the current git commit hash of the repository.
pub fn current_commit_hash() -> Result<String> {
    let output = Command::new("git").args(["rev-parse", "HEAD"]).output()?;
    if !output.status.success() {
        anyhow::bail!(
            "Failed to get git commit hash: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    let hash = String::from_utf8(output.stdout)?;
    Ok(hash.trim().to_string())
}

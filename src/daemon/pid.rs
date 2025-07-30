use anyhow::{Context, Result};
use std::fs;
use std::path::PathBuf;

const PID_FILE: &str = ".wacraft-reminders.pid";

/// Returns the path to the PID file.
fn get_pid_path() -> PathBuf {
    PathBuf::from(PID_FILE)
}

/// Writes the current process ID to the PID file.
pub fn write_pid_file() -> Result<()> {
    let path = get_pid_path();
    let pid = std::process::id();
    fs::write(&path, pid.to_string())
        .with_context(|| format!("Failed to write PID file to {:?}", path))?;
    Ok(())
}

/// Reads the process ID from the PID file.
pub fn read_pid_file() -> Result<u32> {
    let path = get_pid_path();
    let pid_str = fs::read_to_string(&path)
        .with_context(|| format!("Failed to read PID file from {:?}", path))?;
    let pid = pid_str
        .trim()
        .parse::<u32>()
        .context("Failed to parse PID from file")?;
    Ok(pid)
}

/// Removes the PID file.
pub fn remove_pid_file() -> Result<()> {
    let path = get_pid_path();
    if path.exists() {
        fs::remove_file(&path)
            .with_context(|| format!("Failed to remove PID file at {:?}", path))?;
    }
    Ok(())
}

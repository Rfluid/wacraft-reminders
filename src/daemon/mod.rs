use crate::cmd::reminders::send_reminder_to_contact;
use crate::config;
use crate::core::wacraft::client::WacraftClient;
use anyhow::{Context, Result};
use log::info;
use log::{error, warn};
use std::fs::{self, OpenOptions};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use tokio::time::{Duration, interval};
pub mod pid;
use log::LevelFilter;
use simple_logging;

const LOG_FILE: &str = ".wacraft-reminders.log";

/// Configures the logger to write to a file in detached mode,
/// or to the console in foreground mode.
fn setup_logging(detached: bool) -> Result<()> {
    let log_level = LevelFilter::Info;

    if detached {
        // In detached mode, log to a file
        let log_file = OpenOptions::new()
            .create(true)
            .write(true)
            .append(true)
            .open(LOG_FILE)
            .context("Failed to open or create log file")?;

        simple_logging::log_to(log_file, log_level);
    } else {
        // In foreground mode, log directly to the console (stderr)
        simple_logging::log_to_stderr(log_level);
    }

    Ok(())
}

/// The main entry point for the daemon's run logic.
pub async fn run_daemon_process(
    interval_secs: u64,
    batch_size: u32,
    mock: bool,
    detached: bool,
) -> Result<()> {
    setup_logging(detached)?;
    // Write the PID file now that the process is running
    if detached {
        pid::write_pid_file()?;
    }

    info!(
        "Daemon process started. Interval: {}s, Batch Size: {}.",
        interval_secs, batch_size
    );

    let mut timer = interval(Duration::from_secs(interval_secs));

    loop {
        timer.tick().await;
        info!("Daemon tick: Starting reminder processing cycle.");
        if let Err(e) = process_reminders_cycle(batch_size, mock).await {
            error!("Error during reminder processing cycle: {:?}", e);
        }
    }
}

/// Executes a single cycle of fetching all conversations and processing reminders.
async fn process_reminders_cycle(batch_size: u32, mock: bool) -> Result<()> {
    let settings = config::load_settings().context("Daemon: Failed to load settings.json")?;
    let client = WacraftClient::new(settings.wacraft.clone());
    let mut offset = 0;

    loop {
        info!(
            "Fetching conversations batch: limit={}, offset={}",
            batch_size, offset
        );

        let conversations = client
            .get_conversations(batch_size, offset, None)
            .await
            .context("Daemon: Failed to fetch conversations batch")?;

        if conversations.is_empty() {
            info!("No more conversations to process in this cycle.");
            break;
        }

        for conversation in &conversations {
            if let Some(contact) = &conversation.to_contact {
                let contact_id = &contact.id;
                match send_reminder_to_contact(contact_id, &settings, Some(conversation), mock)
                    .await
                {
                    Ok(_) => info!("Successfully processed contact ID: {}", contact_id),
                    Err(e) => warn!(
                        "Failed to process reminder for contact ID {}: {:?}",
                        contact_id, e
                    ),
                }
            }
        }
        offset += batch_size;
    }
    info!("Finished reminder processing cycle.");
    Ok(())
}

/// Detaches the current process to run in the background.
pub fn detach_process(interval_secs: u64, batch_size: u32, mock: bool) -> Result<()> {
    info!("Detaching daemon process...");
    let self_exe = std::env::current_exe().context("Failed to get current executable path")?;

    // Build the arguments vector to pass to the new process.
    let mut args = vec![
        "daemon".to_string(),
        "run".to_string(),
        "--internal-run-detached".to_string(),
        "--interval".to_string(),
        interval_secs.to_string(),
        "--batch-size".to_string(),
        batch_size.to_string(),
    ];

    if mock {
        args.push("--mock".to_string());
    }

    // Re-spawn the process with all the necessary arguments.
    Command::new(self_exe)
        .args(&args)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .context("Failed to spawn detached daemon process")?;

    info!("Daemon process started in the background.");
    Ok(())
}

/// Stops the running daemon process.
pub fn stop_daemon() -> Result<()> {
    let pid = pid::read_pid_file()
        .context("Daemon does not appear to be running (no PID file found).")?;

    info!("Sending SIGTERM to process with PID: {}", pid);

    #[cfg(unix)]
    {
        use nix::sys::signal::{Signal, kill};
        use nix::unistd::Pid;
        kill(Pid::from_raw(pid as i32), Signal::SIGTERM)
            .context("Failed to send SIGTERM to daemon process")?;
    }

    #[cfg(not(unix))]
    {
        // Basic implementation for non-Unix systems like Windows
        // A more robust solution would use platform-specific libraries
        let status = Command::new("taskkill")
            .args(["/PID", &pid.to_string(), "/F"])
            .status()?;
        if !status.success() {
            anyhow::bail!("Failed to kill process with taskkill.");
        }
    }

    pid::remove_pid_file()?;
    info!("Daemon process stopped.");
    Ok(())
}

/// Displays the last few lines of the daemon's log file.
pub fn show_logs() -> Result<()> {
    let log_path = PathBuf::from(LOG_FILE);
    if !log_path.exists() {
        anyhow::bail!("Log file not found. Has the daemon run yet?");
    }
    let content = fs::read_to_string(log_path)?;
    println!("--- Last logs from {} ---", LOG_FILE);
    // Simple implementation: print the whole file. A real-world app might use `tail`.
    println!("{}", content);
    Ok(())
}

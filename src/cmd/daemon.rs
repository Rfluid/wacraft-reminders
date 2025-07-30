use crate::daemon;
use anyhow::Result;
use clap::Subcommand;

/// Actions for running the background daemon.
#[derive(Subcommand, Debug)]
pub enum DaemonAction {
    /// Starts the daemon to periodically check for inactive contacts and send reminders.
    Run {
        /// The interval, in seconds, between each check.
        #[arg(long, default_value = "3600")]
        interval: u64,

        /// The number of conversations to fetch from the API in each batch.
        #[arg(long, default_value = "100")]
        batch_size: u32,

        /// Run the daemon in a detached (background) process.
        #[arg(long)]
        detached: bool,

        /// An internal flag used by the daemon to run the actual process after detaching.
        #[arg(long, hide = true)]
        internal_run_detached: bool,

        /// (Internal) Skips actual message sending, useful for testing.
        #[arg(long, hide = true)]
        mock: bool,
    },
    /// Stops the running daemon process.
    Stop,
    /// Shows the daemon's log output.
    Logs,
}

/// Handles the `daemon` subcommand by dispatching to the appropriate function.
pub async fn handle_daemon_command(action: DaemonAction) -> Result<()> {
    match action {
        DaemonAction::Run {
            interval,
            batch_size,
            detached,
            internal_run_detached,
            mock,
        } => {
            if internal_run_detached {
                // This is the child process, run the actual daemon logic.
                daemon::run_daemon_process(interval, batch_size, mock, true).await?;
            } else if detached {
                // This is the parent process, detach and exit.
                // Pass all relevant arguments to the detach function.
                daemon::detach_process(interval, batch_size, mock)?;
            } else {
                // Run in the foreground.
                println!("Running daemon in foreground. Press Ctrl+C to stop.");
                daemon::run_daemon_process(interval, batch_size, mock, false).await?;
            }
        }
        DaemonAction::Stop => {
            daemon::stop_daemon()?;
        }
        DaemonAction::Logs => {
            daemon::show_logs()?;
        }
    }
    Ok(())
}

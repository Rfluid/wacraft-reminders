use anyhow::Result;
use clap::{Parser, Subcommand};
mod cmd;
mod config;
mod core;
mod daemon;

#[derive(Parser)]
#[command(
    name = "wacraft-reminders",
    version,
    about = "A CLI tool for sending reminders to inactive Wacraft users.",
    long_about = "This CLI allows managing and automating reminders for contacts based on their inactivity time, using the Wacraft API, email, or webhooks.",
    author = "Ruy Vieira <ruy.vieiraneto@gmail.com>"
)]
struct Cli {
    /// Overrides the default path for the reminders configuration file (e.g., reminders.json).
    #[arg(long, value_name = "FILE_PATH")]
    reminders_config: Option<String>,

    /// Overrides the default path for the services settings file (e.g., settings.json).
    #[arg(long, value_name = "FILE_PATH")]
    settings_config: Option<String>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Manage local configuration files.
    Config {
        #[command(subcommand)]
        action: cmd::config::ConfigAction,
    },
    /// Manage and send reminders manually.
    Reminders {
        #[command(subcommand)]
        action: cmd::reminders::RemindersAction,
    },
    /// Run the background daemon for automated tasks.
    Daemon {
        #[command(subcommand)]
        action: cmd::daemon::DaemonAction,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Config { action } => {
            // Initialize the logger so you can control verbosity via RUST_LOG env var.
            env_logger::init();
            cmd::config::handle_config_command(action).await?;
        }
        Commands::Reminders { action } => {
            // Initialize the logger so you can control verbosity via RUST_LOG env var.
            env_logger::init();
            cmd::reminders::handle_reminders_command(action).await?;
        }
        Commands::Daemon { action } => {
            cmd::daemon::handle_daemon_command(action).await?;
        }
    }

    Ok(())
}

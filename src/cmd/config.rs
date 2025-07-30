use crate::config::{
    self,
    models::{EmailConfig, Settings, WacraftConfig},
};
use anyhow::{Context, Result};
use clap::Subcommand;

/// Actions for managing the local configuration files.
#[derive(Subcommand, Debug)]
pub enum ConfigAction {
    /// Creates default configuration files (`settings.json`, `reminders.json`).
    Init {
        /// Overwrite existing configuration files if they exist.
        #[arg(long)]
        force: bool,
    },
    /// Displays the contents of the configuration files.
    View,
    /// Shows the absolute path to the configuration directory.
    Path,
}

/// Handles the `config` subcommand.
pub async fn handle_config_command(action: ConfigAction) -> Result<()> {
    match action {
        ConfigAction::Init { force } => {
            init_config_files(force)?;
        }
        ConfigAction::View => {
            view_config_files()?;
        }
        ConfigAction::Path => {
            let config_dir = config::get_config_dir()?;
            println!("{}", config_dir.display());
        }
    }
    Ok(())
}

/// Creates the default configuration files.
fn init_config_files(force: bool) -> Result<()> {
    let settings_path = config::get_settings_path()?;
    let reminders_path = config::get_reminders_path()?;

    if !force && (settings_path.exists() || reminders_path.exists()) {
        anyhow::bail!("Configuration files already exist. Use --force to overwrite.");
    }

    // Create default settings
    let default_settings = Settings {
        wacraft: WacraftConfig {
            base_url: "https://api.wacraft.com.br".to_string(),
            email: "user@example.com".to_string(),
            password: "your_password".to_string(),
            access_token: None,
            refresh_token: None,
            token_expires_at: None,
        },
        email: EmailConfig {
            smtp_server: "smtp.example.com".to_string(),
            smtp_port: 587,
            smtp_user: "user@example.com".to_string(),
            smtp_password: "your_smtp_password".to_string(),
            from_address: "reminders@wacraft.com".to_string(),
        },
    };

    // Create empty reminders list
    let default_reminders: Vec<config::models::ReminderRule> = Vec::new();

    config::save_settings(&default_settings).context("Failed to write settings.json")?;
    println!(
        "✅ Created default settings file at: {}",
        settings_path.display()
    );

    config::save_reminders(&default_reminders).context("Failed to write reminders.json")?;
    println!(
        "✅ Created empty reminders file at: {}",
        reminders_path.display()
    );

    println!("\nConfiguration initialized! Please edit the files with your credentials.");

    Ok(())
}

/// Prints the content of the configuration files to the console.
fn view_config_files() -> Result<()> {
    println!("--- Settings ---");
    let settings_path = config::get_settings_path()?;
    match config::load_settings() {
        Ok(settings) => {
            let settings_json = serde_json::to_string_pretty(&settings)?;
            println!("{}", settings_json);
        }
        Err(_) => {
            println!(
                "Could not load settings from: {}. Run 'config init' to create it.",
                settings_path.display()
            );
        }
    }

    println!("\n--- Reminders ---");
    let reminders_path = config::get_reminders_path()?;
    match config::load_reminders() {
        Ok(reminders) => {
            let reminders_json = serde_json::to_string_pretty(&reminders)?;
            println!("{}", reminders_json);
        }
        Err(_) => {
            println!(
                "Could not load reminders from: {}. Run 'config init' to create it.",
                reminders_path.display()
            );
        }
    }

    Ok(())
}

use crate::config::models::{ReminderRule, Settings};
use anyhow::{Context, Result};
use std::fs::{self, File, OpenOptions};
use std::io::{BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};

pub mod models;

const CONFIG_DIR_NAME: &str = "wacraft-reminders";
const SETTINGS_FILE_NAME: &str = "settings.json";
const REMINDERS_FILE_NAME: &str = "reminders.json";

/// Returns the path to the application's configuration directory.
/// It creates the directory if it doesn't exist.
/// e.g., ~/.config/wacraft-reminders/ on Linux.
pub fn get_config_dir() -> Result<PathBuf> {
    let config_dir = dirs::config_dir()
        .ok_or_else(|| anyhow::anyhow!("Could not find a valid config directory."))?
        .join(CONFIG_DIR_NAME);

    if !config_dir.exists() {
        fs::create_dir_all(&config_dir).context("Failed to create config directory")?;
    }
    Ok(config_dir)
}

/// Returns the full path to the `settings.json` file.
pub fn get_settings_path() -> Result<PathBuf> {
    Ok(get_config_dir()?.join(SETTINGS_FILE_NAME))
}

/// Returns the full path to the `reminders.json` file.
pub fn get_reminders_path() -> Result<PathBuf> {
    Ok(get_config_dir()?.join(REMINDERS_FILE_NAME))
}

/// A generic function to read and deserialize a JSON file into a given type `T`.
fn read_json_file<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T> {
    let file = File::open(path).with_context(|| format!("Failed to open file: {:?}", path))?;
    let reader = BufReader::new(file);
    let data = serde_json::from_reader(reader)
        .with_context(|| format!("Failed to parse JSON from file: {:?}", path))?;
    Ok(data)
}

/// A generic function to serialize a given type `T` and write it to a JSON file.
fn write_json_file<T: ?Sized + serde::Serialize>(path: &Path, data: &T) -> Result<()> {
    let file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(path)
        .with_context(|| format!("Failed to create or open file for writing: {:?}", path))?;
    let mut writer = BufWriter::new(file);
    serde_json::to_writer_pretty(&mut writer, data)
        .with_context(|| format!("Failed to write JSON to file: {:?}", path))?;
    writer
        .flush()
        .with_context(|| format!("Failed to flush writer for file: {:?}", path))?;
    Ok(())
}

// --- Public API for Configuration Management ---

/// Loads the `Settings` struct from the `settings.json` file.
pub fn load_settings() -> Result<Settings> {
    let path = get_settings_path()?;
    read_json_file(&path)
}

/// Saves the `Settings` struct to the `settings.json` file.
pub fn save_settings(settings: &Settings) -> Result<()> {
    let path = get_settings_path()?;
    write_json_file(&path, settings)
}

/// Loads the list of reminder rules from the `reminders.json` file.
pub fn load_reminders() -> Result<Vec<ReminderRule>> {
    let path = get_reminders_path()?;
    if !path.exists() {
        // If the file doesn't exist, return an empty list.
        return Ok(Vec::new());
    }
    read_json_file(&path)
}

/// Saves a list of reminder rules to the `reminders.json` file.
pub fn save_reminders(reminders: &[ReminderRule]) -> Result<()> {
    let path = get_reminders_path()?;
    write_json_file(&path, reminders)
}

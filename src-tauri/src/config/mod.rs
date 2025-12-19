use rusqlite::{params, Connection, OpenFlags};
use keyring::Entry;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use std::fs;
use tauri::{AppHandle, Manager, Runtime};
use thiserror::Error;

const SERVICE_NAME: &str = "flow-paste";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppConfig {
    pub hotkey: String,
    pub ai_provider: String,
    pub ollama_base_url: String,
    pub openai_base_url: String,
    pub model_name: String,
    pub theme: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            hotkey: "Ctrl+Shift+V".to_string(),
            ai_provider: "Ollama".to_string(),
            ollama_base_url: "http://localhost:11434".to_string(),
            openai_base_url: "https://api.openai.com/v1".to_string(),
            model_name: "llama3.2".to_string(),
            theme: "system".to_string(),
        }
    }
}

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("failed to get config directory")]
    ConfigDirUnavailable,
    #[error("database error: {0}")]
    Database(String),
    #[error("keyring error: {0}")]
    Keyring(String),
    #[error("io error: {0}")]
    Io(String),
}

pub struct ConfigManager {
    db: Mutex<Connection>,
}

impl ConfigManager {
    pub fn init<R: Runtime>(app: &AppHandle<R>) -> Result<Self, ConfigError> {
        let config_dir = app
            .path()
            .app_config_dir()
            .map_err(|_| ConfigError::ConfigDirUnavailable)?;

        if !config_dir.exists() {
            fs::create_dir_all(&config_dir)
                .map_err(|e| ConfigError::Io(e.to_string()))?;
        }

        let db_path = config_dir.join("settings.db");
        let conn = Connection::open_with_flags(
            &db_path,
            OpenFlags::SQLITE_OPEN_READ_WRITE
                | OpenFlags::SQLITE_OPEN_CREATE
                | OpenFlags::SQLITE_OPEN_FULL_MUTEX,
        )
        .map_err(|e| ConfigError::Database(e.to_string()))?;

        // Enable WAL mode for better concurrency
        conn.pragma_update(None, "journal_mode", "WAL")
            .map_err(|e| ConfigError::Database(e.to_string()))?;

        // Create settings table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS settings (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            )",
            [],
        )
        .map_err(|e| ConfigError::Database(e.to_string()))?;

        log::info!("Config manager initialized at {:?}", db_path);

        Ok(Self {
            db: Mutex::new(conn),
        })
    }

    pub fn get_config(&self) -> Result<AppConfig, ConfigError> {
        let conn = self
            .db
            .lock()
            .map_err(|_| ConfigError::Database("database lock poisoned".into()))?;

        let mut stmt = conn
            .prepare("SELECT key, value FROM settings")
            .map_err(|e| ConfigError::Database(e.to_string()))?;

        let rows = stmt
            .query_map([], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
            })
            .map_err(|e| ConfigError::Database(e.to_string()))?;

        let mut config = AppConfig::default();

        for row in rows {
            let (key, value) = row.map_err(|e| ConfigError::Database(e.to_string()))?;
            match key.as_str() {
                "hotkey" => config.hotkey = value,
                "aiProvider" => config.ai_provider = value,
                "ollamaBaseUrl" => config.ollama_base_url = value,
                "openaiBaseUrl" => config.openai_base_url = value,
                "modelName" => config.model_name = value,
                "theme" => config.theme = value,
                _ => {}
            }
        }

        Ok(config)
    }

    pub fn set_config(&self, config: &AppConfig) -> Result<(), ConfigError> {
        let conn = self
            .db
            .lock()
            .map_err(|_| ConfigError::Database("database lock poisoned".into()))?;

        let pairs = [
            ("hotkey", &config.hotkey),
            ("aiProvider", &config.ai_provider),
            ("ollamaBaseUrl", &config.ollama_base_url),
            ("openaiBaseUrl", &config.openai_base_url),
            ("modelName", &config.model_name),
            ("theme", &config.theme),
        ];

        for (key, value) in pairs {
            conn.execute(
                "INSERT OR REPLACE INTO settings (key, value) VALUES (?1, ?2)",
                params![key, value],
            )
            .map_err(|e| ConfigError::Database(e.to_string()))?;
        }

        Ok(())
    }

    pub fn get_api_key(&self, provider: &str) -> Result<Option<String>, ConfigError> {
        let entry = Entry::new(SERVICE_NAME, provider)
            .map_err(|e| ConfigError::Keyring(e.to_string()))?;

        match entry.get_password() {
            Ok(pwd) => Ok(Some(pwd)),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(e) => Err(ConfigError::Keyring(e.to_string())),
        }
    }

    pub fn set_api_key(&self, provider: &str, key: &str) -> Result<(), ConfigError> {
        let entry = Entry::new(SERVICE_NAME, provider)
            .map_err(|e| ConfigError::Keyring(e.to_string()))?;

        if key.is_empty() {
            // Delete the key if empty string provided
            match entry.delete_credential() {
                Ok(_) => Ok(()),
                Err(keyring::Error::NoEntry) => Ok(()), // Already deleted
                Err(e) => Err(ConfigError::Keyring(e.to_string())),
            }
        } else {
            entry
                .set_password(key)
                .map_err(|e| ConfigError::Keyring(e.to_string()))
        }
    }
}

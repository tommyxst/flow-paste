use rusqlite::{params, Connection, OpenFlags};
use keyring::Entry;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use std::fs;
use tauri::{AppHandle, Manager, Runtime};
use thiserror::Error;
use crate::regex::Rule;

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
    #[serde(default)]
    pub pinned_rule_ids: Vec<String>,
    #[serde(default)]
    pub custom_rules: Vec<Rule>,
    #[serde(default = "default_true")]
    pub enable_ai_rule_learning: bool,
}

fn default_true() -> bool { true }

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            hotkey: "Ctrl+Shift+V".to_string(),
            ai_provider: "Ollama".to_string(),
            ollama_base_url: "http://localhost:11434".to_string(),
            openai_base_url: "https://api.openai.com/v1".to_string(),
            model_name: "llama3.2".to_string(),
            theme: "system".to_string(),
            pinned_rule_ids: vec![
                "remove_spaces".to_string(),
                "remove_empty_lines".to_string(),
                "format_json".to_string(),
            ],
            custom_rules: vec![],
            enable_ai_rule_learning: true,
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

        // Set busy timeout to handle concurrent access (5 seconds)
        conn.pragma_update(None, "busy_timeout", "5000")
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
                "pinnedRuleIds" => {
                    if let Ok(val) = serde_json::from_str(&value) {
                         config.pinned_rule_ids = val;
                    }
                },
                "customRules" => {
                    if let Ok(val) = serde_json::from_str(&value) {
                         config.custom_rules = val;
                    }
                },
                "enableAIRuleLearning" => {
                    config.enable_ai_rule_learning = value == "true";
                },
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

        // Special handling for complex types (Vecs) to store as JSON strings in SQLite
        let pinned_rules_json = serde_json::to_string(&config.pinned_rule_ids).unwrap_or_default();
        let custom_rules_json = serde_json::to_string(&config.custom_rules).unwrap_or_default();
        
        // Execute simple fields
        for (key, value) in pairs {
            conn.execute(
                "INSERT OR REPLACE INTO settings (key, value) VALUES (?1, ?2)",
                params![key, value],
            )
            .map_err(|e| ConfigError::Database(e.to_string()))?;
        }

        // Execute complex fields
        let complex_pairs = [
            ("pinnedRuleIds", pinned_rules_json),
            ("customRules", custom_rules_json),
            ("enableAIRuleLearning", config.enable_ai_rule_learning.to_string()),
        ];

        for (key, value) in complex_pairs {
             conn.execute(
                "INSERT OR REPLACE INTO settings (key, value) VALUES (?1, ?2)",
                params![key, value],
            )
            .map_err(|e| ConfigError::Database(e.to_string()))?;
        }

        Ok(())
    }

    pub fn get_api_key(&self, provider: &str) -> Result<Option<String>, ConfigError> {
        log::debug!("get_api_key: trying keyring for provider='{}'", provider);
        
        // Try keyring first
        if let Ok(entry) = Entry::new(SERVICE_NAME, provider) {
            if let Ok(pwd) = entry.get_password() {
                log::debug!("get_api_key: found in keyring, len={}", pwd.len());
                return Ok(Some(pwd));
            }
        }
        
        // Fallback to SQLite
        log::debug!("get_api_key: keyring failed, trying SQLite fallback");
        let conn = self.db.lock().map_err(|e| ConfigError::Database(e.to_string()))?;
        let result: Result<String, _> = conn.query_row(
            "SELECT value FROM api_keys WHERE provider = ?",
            params![provider],
            |row| row.get(0),
        );
        
        match result {
            Ok(key) => {
                log::debug!("get_api_key: found in SQLite, len={}", key.len());
                Ok(Some(key))
            }
            Err(rusqlite::Error::QueryReturnedNoRows) => {
                log::debug!("get_api_key: not found in SQLite");
                Ok(None)
            }
            Err(e) => {
                log::error!("get_api_key: SQLite error: {}", e);
                Err(ConfigError::Database(e.to_string()))
            }
        }
    }

    pub fn set_api_key(&self, provider: &str, key: &str) -> Result<(), ConfigError> {
        log::debug!("set_api_key: provider='{}', key_len={}", provider, key.len());
        
        // Try keyring first
        let keyring_ok = if let Ok(entry) = Entry::new(SERVICE_NAME, provider) {
            if key.is_empty() {
                entry.delete_credential().is_ok()
            } else {
                entry.set_password(key).is_ok()
            }
        } else {
            false
        };
        
        if keyring_ok {
            log::debug!("set_api_key: saved to keyring successfully");
        } else {
            log::warn!("set_api_key: keyring failed, using SQLite fallback");
        }
        
        // Always save to SQLite as fallback
        let conn = self.db.lock().map_err(|e| ConfigError::Database(e.to_string()))?;
        
        // Ensure table exists
        conn.execute(
            "CREATE TABLE IF NOT EXISTS api_keys (provider TEXT PRIMARY KEY, value TEXT NOT NULL)",
            [],
        ).map_err(|e| ConfigError::Database(e.to_string()))?;
        
        if key.is_empty() {
            conn.execute("DELETE FROM api_keys WHERE provider = ?", params![provider])
                .map_err(|e| ConfigError::Database(e.to_string()))?;
            log::debug!("set_api_key: deleted from SQLite");
        } else {
            conn.execute(
                "INSERT OR REPLACE INTO api_keys (provider, value) VALUES (?, ?)",
                params![provider, key],
            ).map_err(|e| ConfigError::Database(e.to_string()))?;
            log::debug!("set_api_key: saved to SQLite successfully");
        }
        
        Ok(())
    }
}

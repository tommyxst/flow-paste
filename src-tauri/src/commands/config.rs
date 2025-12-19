use tauri::State;

use crate::config::{AppConfig, ConfigManager};

#[tauri::command]
pub async fn get_config(state: State<'_, ConfigManager>) -> Result<AppConfig, String> {
    state.get_config().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn set_config(
    config: AppConfig,
    state: State<'_, ConfigManager>,
) -> Result<(), String> {
    state.set_config(&config).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_api_key(
    provider: String,
    state: State<'_, ConfigManager>,
) -> Result<Option<String>, String> {
    state.get_api_key(&provider).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn set_api_key(
    provider: String,
    key: String,
    state: State<'_, ConfigManager>,
) -> Result<(), String> {
    state.set_api_key(&provider, &key).map_err(|e| e.to_string())
}

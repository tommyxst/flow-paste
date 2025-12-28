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
    let result = state.get_api_key(&provider);
    log::info!("get_api_key('{}') => has_key: {}", provider, result.as_ref().map(|r| r.is_some()).unwrap_or(false));
    result.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn set_api_key(
    provider: String,
    key: String,
    state: State<'_, ConfigManager>,
) -> Result<(), String> {
    log::info!("set_api_key('{}', key_len={})", provider, key.len());
    let result = state.set_api_key(&provider, &key);
    log::info!("set_api_key result: {:?}", result.is_ok());
    result.map_err(|e| e.to_string())
}

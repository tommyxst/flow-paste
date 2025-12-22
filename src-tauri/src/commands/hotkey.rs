use tauri::{AppHandle, State};
use crate::hotkey::HotkeyManager;

#[tauri::command]
pub async fn register_hotkey(
    app: AppHandle,
    hotkey: String,
    manager: State<'_, HotkeyManager>,
) -> Result<(), String> {
    manager.register_hotkey(&app, &hotkey)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn unregister_hotkey(
    app: AppHandle,
    manager: State<'_, HotkeyManager>,
) -> Result<(), String> {
    manager.unregister_hotkey(&app)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn is_hotkey_registered(
    manager: State<'_, HotkeyManager>,
) -> Result<bool, String> {
    Ok(manager.is_registered().await)
}

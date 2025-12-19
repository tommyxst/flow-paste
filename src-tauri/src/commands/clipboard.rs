use tauri::AppHandle;

use crate::clipboard::{self, ClipboardContent};

#[tauri::command]
pub async fn read_clipboard(app: AppHandle) -> Result<ClipboardContent, String> {
    let result = tauri::async_runtime::spawn_blocking(move || clipboard::read_clipboard(&app))
        .await
        .map_err(|e| e.to_string())?;
    result.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn write_clipboard(app: AppHandle, text: String) -> Result<(), String> {
    let result = tauri::async_runtime::spawn_blocking(move || clipboard::write_clipboard(&app, &text))
        .await
        .map_err(|e| e.to_string())?;
    result.map_err(|e| e.to_string())
}

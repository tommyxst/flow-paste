use serde::Serialize;
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

/// Paste result for frontend to handle fallback
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PasteResult {
    pub success: bool,
    pub used_simulation: bool,
    pub message: Option<String>,
}

/// Paste text to cursor position with fallback to clipboard-only
///
/// Attempts keyboard simulation (Ctrl/Cmd+V), falls back to clipboard write on failure.
#[tauri::command]
pub async fn paste_to_cursor(
    app: AppHandle,
    text: String,
    delay_ms: Option<u64>,
) -> Result<PasteResult, String> {
    let result = tauri::async_runtime::spawn_blocking(move || {
        // First check platform capability
        if let Err(e) = clipboard::check_paste_capability() {
            log::warn!("Paste capability check failed: {}, falling back to clipboard", e);
            return match clipboard::write_clipboard(&app, &text) {
                Ok(()) => PasteResult {
                    success: true,
                    used_simulation: false,
                    message: Some(format!("Copied to clipboard ({}). Press Ctrl/Cmd+V to paste.", e)),
                },
                Err(e) => PasteResult {
                    success: false,
                    used_simulation: false,
                    message: Some(format!("Failed to write to clipboard: {}", e)),
                },
            };
        }

        // Try paste with simulation
        match clipboard::paste_to_cursor(&app, &text, delay_ms) {
            Ok(()) => PasteResult {
                success: true,
                used_simulation: true,
                message: None,
            },
            Err(e) => {
                // All errors except ClipboardWrite should fall back to clipboard
                log::warn!("Paste simulation failed: {}, falling back to clipboard", e);
                match clipboard::write_clipboard(&app, &text) {
                    Ok(()) => PasteResult {
                        success: true,
                        used_simulation: false,
                        message: Some(format!(
                            "Copied to clipboard. Press Ctrl/Cmd+V to paste. ({})",
                            e
                        )),
                    },
                    Err(write_err) => PasteResult {
                        success: false,
                        used_simulation: false,
                        message: Some(format!("Failed to write to clipboard: {}", write_err)),
                    },
                }
            }
        }
    })
    .await
    .map_err(|e| e.to_string())?;

    Ok(result)
}

/// Check if paste simulation is available on this platform
#[tauri::command]
pub fn check_paste_capability() -> Result<(), String> {
    clipboard::check_paste_capability().map_err(|e| e.to_string())
}

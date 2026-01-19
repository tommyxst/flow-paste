//! Clipboard History Commands
//!
//! Tauri commands for clipboard history management.

use std::sync::Arc;
use tauri::{AppHandle, State};

use crate::clipboard::{
    history::ClipboardHistoryService, ClipboardHistoryEntry, ClipboardHistoryItem,
};

use super::PasteResult;

#[tauri::command]
pub async fn list_clipboard_history(
    state: State<'_, Arc<ClipboardHistoryService>>,
    limit: Option<u32>,
    offset: Option<u32>,
) -> Result<Vec<ClipboardHistoryEntry>, String> {
    // Clamp limit to prevent abuse (max 100, default 50)
    let limit = limit.unwrap_or(50).min(100) as usize;
    // Clamp offset to reasonable range (max 10000)
    let offset = offset.unwrap_or(0).min(10000) as usize;

    let service = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || service.list(limit, offset))
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_clipboard_history(
    state: State<'_, Arc<ClipboardHistoryService>>,
    id: i64,
) -> Result<ClipboardHistoryItem, String> {
    let service = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || service.get(id))
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_clipboard_history(
    state: State<'_, Arc<ClipboardHistoryService>>,
    id: i64,
) -> Result<(), String> {
    let service = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || service.delete(id))
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn clear_clipboard_history(
    app: AppHandle,
    state: State<'_, Arc<ClipboardHistoryService>>,
) -> Result<(), String> {
    let service = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || service.clear())
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())?;

    // Notify poller to reset its last_hash
    use tauri::Emitter;
    let _ = app.emit("clipboard:history_cleared", ());

    Ok(())
}

#[tauri::command]
pub async fn paste_clipboard_history(
    app: AppHandle,
    state: State<'_, Arc<ClipboardHistoryService>>,
    id: i64,
    delay_ms: Option<u64>,
) -> Result<PasteResult, String> {
    use crate::clipboard;

    // Get the history item
    let service = state.inner().clone();
    let item = tauri::async_runtime::spawn_blocking(move || service.get(id))
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())?;

    // Only text items can be pasted
    let text = item.text.ok_or_else(|| "History item has no text content".to_string())?;

    // Use existing paste logic
    let result = tauri::async_runtime::spawn_blocking(move || {
        if let Err(e) = clipboard::check_paste_capability() {
            log::warn!("Paste capability check failed: {}, falling back to clipboard", e);
            return match clipboard::write_clipboard(&app, &text) {
                Ok(()) => PasteResult {
                    success: true,
                    used_simulation: false,
                    message: Some(format!(
                        "Copied to clipboard ({}). Press Ctrl/Cmd+V to paste.",
                        e
                    )),
                },
                Err(e) => PasteResult {
                    success: false,
                    used_simulation: false,
                    message: Some(format!("Failed to write to clipboard: {}", e)),
                },
            };
        }

        match clipboard::paste_to_cursor(&app, &text, delay_ms) {
            Ok(()) => PasteResult {
                success: true,
                used_simulation: true,
                message: None,
            },
            Err(e) => {
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

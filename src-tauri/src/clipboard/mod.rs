use serde::{Deserialize, Serialize};
use tauri::AppHandle;
use tauri_plugin_clipboard_manager::ClipboardExt;
use thiserror::Error;

pub mod history;
pub mod paste;
pub mod poller;

pub use history::{ClipboardHistoryEntry, ClipboardHistoryItem, ClipboardHistoryService};
pub use paste::{check_paste_capability, paste_to_cursor};
pub use poller::start_clipboard_poller;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ClipboardKind {
    Text,
    Image,
    Unknown,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ClipboardImageMeta {
    pub width: u32,
    pub height: u32,
    pub byte_length: usize,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ClipboardContent {
    pub kind: ClipboardKind,
    pub text: Option<String>,
    pub image: Option<ClipboardImageMeta>,
}

#[derive(Debug, Error)]
pub enum ClipboardError {
    #[error("clipboard unavailable: {0}")]
    Unavailable(String),
    #[error("clipboard is empty")]
    Empty,
    #[error("unsupported clipboard content")]
    Unsupported,
}

pub fn read_clipboard(app: &AppHandle) -> Result<ClipboardContent, ClipboardError> {
    let clipboard = app.clipboard();

    // Try text first (most common)
    match clipboard.read_text() {
        Ok(text) => {
            return Ok(ClipboardContent {
                kind: ClipboardKind::Text,
                text: Some(text),
                image: None,
            });
        }
        Err(e) => {
            log::debug!("Failed to read text from clipboard: {}", e);
        }
    }

    // Try image
    match clipboard.read_image() {
        Ok(image) => {
            return Ok(ClipboardContent {
                kind: ClipboardKind::Image,
                text: None,
                image: Some(ClipboardImageMeta {
                    width: image.width(),
                    height: image.height(),
                    byte_length: image.rgba().len(),
                }),
            });
        }
        Err(e) => {
            log::debug!("Failed to read image from clipboard: {}", e);
        }
    }

    // Clipboard is empty or has unsupported content - return Unknown instead of error
    // This is a valid state, not an error condition
    Ok(ClipboardContent {
        kind: ClipboardKind::Unknown,
        text: None,
        image: None,
    })
}

pub fn write_clipboard(app: &AppHandle, text: &str) -> Result<(), ClipboardError> {
    if text.is_empty() {
        return Err(ClipboardError::Empty);
    }

    app.clipboard()
        .write_text(text)
        .map_err(|e| ClipboardError::Unavailable(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clipboard_kind_serialize() {
        assert_eq!(
            serde_json::to_string(&ClipboardKind::Text).unwrap(),
            "\"text\""
        );
        assert_eq!(
            serde_json::to_string(&ClipboardKind::Image).unwrap(),
            "\"image\""
        );
    }
}

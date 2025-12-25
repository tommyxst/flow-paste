//! Paste-to-cursor functionality
//!
//! Implements keyboard simulation to paste content directly at cursor position.
//! Based on Codex review feedback:
//! - Proper error handling with distinct error types
//! - Safe modifier key release using RAII pattern
//! - Platform-specific handling (Windows/macOS)
//! - Configurable delay for clipboard sync

use enigo::{Enigo, Key, Keyboard, Settings};
use serde::Serialize;
use std::thread;
use std::time::Duration;
use tauri::AppHandle;
use thiserror::Error;

use super::{write_clipboard, ClipboardError};

/// Delay between clipboard write and paste simulation (ms)
const DEFAULT_PASTE_DELAY_MS: u64 = 100;

#[derive(Debug, Error, Serialize)]
pub enum PasteError {
    #[error("Failed to write to clipboard: {0}")]
    ClipboardWrite(String),

    #[error("Keyboard simulation failed: {0}")]
    SimulationFailed(String),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("Platform not supported for paste simulation")]
    PlatformNotSupported,
}

impl From<ClipboardError> for PasteError {
    fn from(err: ClipboardError) -> Self {
        PasteError::ClipboardWrite(err.to_string())
    }
}

/// RAII guard to ensure modifier keys are released
struct ModifierGuard {
    enigo: Option<Enigo>,
    modifier: Key,
    pressed: bool,
}

impl ModifierGuard {
    fn new(mut enigo: Enigo, modifier: Key) -> Result<Self, PasteError> {
        // Try to press the modifier key
        if let Err(e) = enigo.key(modifier, enigo::Direction::Press) {
            return Err(PasteError::SimulationFailed(format!(
                "Failed to press modifier key: {:?}",
                e
            )));
        }
        Ok(Self {
            enigo: Some(enigo),
            modifier,
            pressed: true,
        })
    }

    fn click_key(&mut self, key: Key) -> Result<(), PasteError> {
        if let Some(ref mut enigo) = self.enigo {
            enigo
                .key(key, enigo::Direction::Click)
                .map_err(|e| PasteError::SimulationFailed(format!("Failed to click key: {:?}", e)))
        } else {
            Err(PasteError::SimulationFailed("Enigo not available".to_string()))
        }
    }

    fn release(&mut self) -> Result<(), PasteError> {
        if self.pressed {
            if let Some(ref mut enigo) = self.enigo {
                if let Err(e) = enigo.key(self.modifier, enigo::Direction::Release) {
                    log::warn!("Failed to release modifier key: {:?}", e);
                }
            }
            self.pressed = false;
        }
        Ok(())
    }
}

impl Drop for ModifierGuard {
    fn drop(&mut self) {
        if self.pressed {
            if let Some(ref mut enigo) = self.enigo {
                if let Err(e) = enigo.key(self.modifier, enigo::Direction::Release) {
                    log::error!("Failed to release modifier key in drop: {:?}", e);
                }
            }
            self.pressed = false;
        }
    }
}

/// Paste text to cursor by writing to clipboard and simulating Ctrl/Cmd+V
///
/// # Arguments
/// * `app` - Tauri app handle for clipboard access
/// * `text` - Text to paste
/// * `delay_ms` - Optional delay in ms before simulating paste (default: 100ms)
///
/// # Returns
/// * `Ok(())` on success
/// * `Err(PasteError)` with specific error type on failure
pub fn paste_to_cursor(
    app: &AppHandle,
    text: &str,
    delay_ms: Option<u64>,
) -> Result<(), PasteError> {
    // Step 1: Write to clipboard
    write_clipboard(app, text)?;

    // Step 2: Wait for clipboard to sync
    let delay = delay_ms.unwrap_or(DEFAULT_PASTE_DELAY_MS);
    thread::sleep(Duration::from_millis(delay));

    // Step 3: Simulate paste keystroke
    simulate_paste()?;

    log::info!("Successfully pasted {} chars to cursor", text.len());
    Ok(())
}

/// Simulate Ctrl+V (Windows/Linux) or Cmd+V (macOS) keystroke
fn simulate_paste() -> Result<(), PasteError> {
    // Create Enigo instance
    let enigo = Enigo::new(&Settings::default()).map_err(|e| {
        let msg = format!("Failed to initialize keyboard simulation: {:?}", e);

        // Check for permission-related errors
        if cfg!(target_os = "macos") && msg.contains("accessibility") {
            PasteError::PermissionDenied(
                "macOS Accessibility permission required. Please enable it in System Preferences > Security & Privacy > Privacy > Accessibility".to_string()
            )
        } else {
            PasteError::SimulationFailed(msg)
        }
    })?;

    // Determine the correct modifier key for the platform
    #[cfg(target_os = "macos")]
    let modifier = Key::Meta;

    #[cfg(not(target_os = "macos"))]
    let modifier = Key::Control;

    // Use RAII guard to ensure modifier is released
    let mut guard = ModifierGuard::new(enigo, modifier)?;

    // Click 'V' key
    guard.click_key(Key::Unicode('v'))?;

    // Explicitly release modifier
    guard.release()?;

    Ok(())
}

/// Check if paste simulation is likely to work on this platform
pub fn check_paste_capability() -> Result<(), PasteError> {
    #[cfg(target_os = "macos")]
    {
        // On macOS, try to create Enigo to check accessibility permission
        match Enigo::new(&Settings::default()) {
            Ok(_) => Ok(()),
            Err(e) => {
                let msg = format!("{:?}", e);
                if msg.contains("accessibility") || msg.contains("permission") {
                    Err(PasteError::PermissionDenied(
                        "macOS requires Accessibility permission for paste simulation. \
                         Enable it in System Preferences > Security & Privacy > Privacy > Accessibility"
                            .to_string(),
                    ))
                } else {
                    Err(PasteError::SimulationFailed(msg))
                }
            }
        }
    }

    #[cfg(target_os = "windows")]
    {
        // Windows generally allows keyboard simulation without special permissions
        // but may fail on elevated/UAC windows
        Ok(())
    }

    #[cfg(target_os = "linux")]
    {
        // Linux X11 generally works, Wayland has restrictions
        if std::env::var("WAYLAND_DISPLAY").is_ok() {
            Err(PasteError::PermissionDenied(
                "Wayland session detected. Paste simulation may not work. \
                 Consider using XWayland or clipboard-only mode."
                    .to_string(),
            ))
        } else {
            Ok(())
        }
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
    {
        Err(PasteError::PlatformNotSupported)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_paste_error_serialization() {
        let err = PasteError::PermissionDenied("test".to_string());
        let json = serde_json::to_string(&err).unwrap();
        assert!(json.contains("PermissionDenied"));
    }
}

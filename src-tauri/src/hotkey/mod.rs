use std::sync::Arc;
use tauri::{AppHandle, Emitter, Manager, Runtime};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};
use tokio::sync::Mutex;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum HotkeyError {
    #[error("invalid hotkey format: {0}")]
    InvalidFormat(String),
    #[error("hotkey registration failed: {0}")]
    RegistrationFailed(String),
}

pub struct HotkeyManager {
    // Use Mutex for exclusive access to registration/unregistration
    current_shortcut: Arc<Mutex<Option<Shortcut>>>,
}

impl HotkeyManager {
    pub fn new() -> Self {
        Self {
            current_shortcut: Arc::new(Mutex::new(None)),
        }
    }

    /// Parse hotkey string like "Ctrl+Shift+V" or "CommandOrControl+Shift+V"
    pub fn parse_hotkey(hotkey: &str) -> Result<Shortcut, HotkeyError> {
        let parts: Vec<&str> = hotkey.split('+').map(|s| s.trim()).collect();

        if parts.is_empty() {
            return Err(HotkeyError::InvalidFormat("Empty hotkey string".to_string()));
        }

        let mut modifiers = Modifiers::empty();
        let key_str = parts.last().unwrap();

        // Validate at least one key
        if key_str.is_empty() {
            return Err(HotkeyError::InvalidFormat("Missing key".to_string()));
        }

        for part in &parts[..parts.len() - 1] {
            match part.to_lowercase().as_str() {
                "ctrl" | "control" => modifiers |= Modifiers::CONTROL,
                "shift" => modifiers |= Modifiers::SHIFT,
                "alt" | "option" => modifiers |= Modifiers::ALT,
                "meta" | "super" | "cmd" | "command" => modifiers |= Modifiers::META,
                "commandorcontrol" | "cmdorctrl" => {
                    #[cfg(target_os = "macos")]
                    {
                        modifiers |= Modifiers::META;
                    }
                    #[cfg(not(target_os = "macos"))]
                    {
                        modifiers |= Modifiers::CONTROL;
                    }
                }
                _ => return Err(HotkeyError::InvalidFormat(format!("Unknown modifier: {}", part))),
            }
        }

        let key = parse_key_code(key_str)?;
        Ok(Shortcut::new(Some(modifiers), key))
    }

    /// Register a hotkey with the given accelerator string (atomic operation)
    pub async fn register_hotkey<R: Runtime>(
        &self,
        app: &AppHandle<R>,
        hotkey: &str,
    ) -> Result<(), HotkeyError> {
        // Parse and validate BEFORE making any changes
        let new_shortcut = Self::parse_hotkey(hotkey)?;

        // Exclusive lock for atomic registration
        let mut current = self.current_shortcut.lock().await;

        // Unregister old hotkey if exists
        if let Some(old_shortcut) = current.take() {
            if let Err(e) = app.global_shortcut().unregister(old_shortcut) {
                log::warn!("Failed to unregister old hotkey: {}", e);
                // Restore old shortcut in state
                *current = Some(old_shortcut);
                // Continue anyway to attempt new registration
            }
        }

        let window = app.get_webview_window("main")
            .ok_or_else(|| HotkeyError::RegistrationFailed("Main window not found".to_string()))?;

        let window_clone = window.clone();

        // Register new hotkey
        app.global_shortcut()
            .on_shortcut(new_shortcut, move |_app, _shortcut, event| {
                if event.state != ShortcutState::Pressed {
                    return;
                }

                log::info!("Global hotkey triggered");

                match window_clone.is_visible() {
                    Ok(visible) => {
                        if visible {
                            if let Err(e) = window_clone.hide() {
                                log::error!("Failed to hide window: {}", e);
                            }
                        } else {
                            if let Err(e) = window_clone.show() {
                                log::error!("Failed to show window: {}", e);
                            }
                            if let Err(e) = window_clone.set_focus() {
                                log::error!("Failed to focus window: {}", e);
                            }
                            if let Err(e) = window_clone.emit("panel:show", ()) {
                                log::error!("Failed to emit panel:show event: {}", e);
                            }
                        }
                    }
                    Err(e) => {
                        log::error!("Failed to check window visibility: {}", e);
                    }
                }
            })
            .map_err(|e| {
                // Registration failed, try to restore old state if available
                log::error!("Failed to register new hotkey: {}", e);
                HotkeyError::RegistrationFailed(e.to_string())
            })?;

        // Update state only after successful registration
        *current = Some(new_shortcut);

        log::info!("Hotkey registered: {}", hotkey);
        Ok(())
    }

    /// Unregister the current hotkey
    pub async fn unregister_hotkey<R: Runtime>(
        &self,
        app: &AppHandle<R>,
    ) -> Result<(), HotkeyError> {
        let mut current = self.current_shortcut.lock().await;

        if let Some(shortcut) = &*current {
            app.global_shortcut()
                .unregister(*shortcut)
                .map_err(|e| HotkeyError::RegistrationFailed(e.to_string()))?;

            // Only clear state after successful unregistration
            *current = None;
            log::info!("Hotkey unregistered");
        }

        Ok(())
    }

    /// Check if a hotkey is currently registered
    pub async fn is_registered(&self) -> bool {
        self.current_shortcut.lock().await.is_some()
    }
}

fn parse_key_code(key: &str) -> Result<Code, HotkeyError> {
    match key.to_uppercase().as_str() {
        // Letters
        "A" => Ok(Code::KeyA),
        "B" => Ok(Code::KeyB),
        "C" => Ok(Code::KeyC),
        "D" => Ok(Code::KeyD),
        "E" => Ok(Code::KeyE),
        "F" => Ok(Code::KeyF),
        "G" => Ok(Code::KeyG),
        "H" => Ok(Code::KeyH),
        "I" => Ok(Code::KeyI),
        "J" => Ok(Code::KeyJ),
        "K" => Ok(Code::KeyK),
        "L" => Ok(Code::KeyL),
        "M" => Ok(Code::KeyM),
        "N" => Ok(Code::KeyN),
        "O" => Ok(Code::KeyO),
        "P" => Ok(Code::KeyP),
        "Q" => Ok(Code::KeyQ),
        "R" => Ok(Code::KeyR),
        "S" => Ok(Code::KeyS),
        "T" => Ok(Code::KeyT),
        "U" => Ok(Code::KeyU),
        "V" => Ok(Code::KeyV),
        "W" => Ok(Code::KeyW),
        "X" => Ok(Code::KeyX),
        "Y" => Ok(Code::KeyY),
        "Z" => Ok(Code::KeyZ),

        // Numbers
        "0" => Ok(Code::Digit0),
        "1" => Ok(Code::Digit1),
        "2" => Ok(Code::Digit2),
        "3" => Ok(Code::Digit3),
        "4" => Ok(Code::Digit4),
        "5" => Ok(Code::Digit5),
        "6" => Ok(Code::Digit6),
        "7" => Ok(Code::Digit7),
        "8" => Ok(Code::Digit8),
        "9" => Ok(Code::Digit9),

        // Function keys
        "F1" => Ok(Code::F1),
        "F2" => Ok(Code::F2),
        "F3" => Ok(Code::F3),
        "F4" => Ok(Code::F4),
        "F5" => Ok(Code::F5),
        "F6" => Ok(Code::F6),
        "F7" => Ok(Code::F7),
        "F8" => Ok(Code::F8),
        "F9" => Ok(Code::F9),
        "F10" => Ok(Code::F10),
        "F11" => Ok(Code::F11),
        "F12" => Ok(Code::F12),

        // Special keys
        "SPACE" => Ok(Code::Space),
        "ENTER" | "RETURN" => Ok(Code::Enter),
        "TAB" => Ok(Code::Tab),
        "BACKSPACE" => Ok(Code::Backspace),
        "ESCAPE" | "ESC" => Ok(Code::Escape),
        "DELETE" | "DEL" => Ok(Code::Delete),
        "INSERT" | "INS" => Ok(Code::Insert),
        "HOME" => Ok(Code::Home),
        "END" => Ok(Code::End),
        "PAGEUP" | "PGUP" => Ok(Code::PageUp),
        "PAGEDOWN" | "PGDN" => Ok(Code::PageDown),

        // Arrow keys
        "UP" | "ARROWUP" => Ok(Code::ArrowUp),
        "DOWN" | "ARROWDOWN" => Ok(Code::ArrowDown),
        "LEFT" | "ARROWLEFT" => Ok(Code::ArrowLeft),
        "RIGHT" | "ARROWRIGHT" => Ok(Code::ArrowRight),

        // Punctuation and symbols
        ";" | "SEMICOLON" => Ok(Code::Semicolon),
        "=" | "EQUAL" | "EQUALS" => Ok(Code::Equal),
        "," | "COMMA" => Ok(Code::Comma),
        "-" | "MINUS" => Ok(Code::Minus),
        "." | "PERIOD" => Ok(Code::Period),
        "/" | "SLASH" => Ok(Code::Slash),
        "`" | "BACKQUOTE" | "BACKTICK" => Ok(Code::Backquote),
        "[" | "BRACKETLEFT" => Ok(Code::BracketLeft),
        "\\" | "BACKSLASH" => Ok(Code::Backslash),
        "]" | "BRACKETRIGHT" => Ok(Code::BracketRight),
        "'" | "QUOTE" => Ok(Code::Quote),

        _ => Err(HotkeyError::InvalidFormat(format!("Unknown key: {}", key))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_hotkey_ctrl_shift_v() {
        let result = HotkeyManager::parse_hotkey("Ctrl+Shift+V");
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_hotkey_command_or_control() {
        let result = HotkeyManager::parse_hotkey("CommandOrControl+Shift+V");
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_hotkey_function_key() {
        let result = HotkeyManager::parse_hotkey("Ctrl+F1");
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_hotkey_punctuation() {
        assert!(HotkeyManager::parse_hotkey("Ctrl+.").is_ok());
        assert!(HotkeyManager::parse_hotkey("Ctrl+/").is_ok());
        assert!(HotkeyManager::parse_hotkey("Ctrl+;").is_ok());
    }

    #[test]
    fn test_parse_hotkey_invalid_modifier() {
        let result = HotkeyManager::parse_hotkey("Invalid+V");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_hotkey_invalid_key() {
        let result = HotkeyManager::parse_hotkey("Ctrl+InvalidKey");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_hotkey_missing_key() {
        let result = HotkeyManager::parse_hotkey("Ctrl+");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_key_code() {
        assert!(parse_key_code("V").is_ok());
        assert!(parse_key_code("F1").is_ok());
        assert!(parse_key_code("Space").is_ok());
        assert!(parse_key_code(".").is_ok());
        assert!(parse_key_code("/").is_ok());
        assert!(parse_key_code("InvalidKey").is_err());
    }
}

use tauri::{Manager, Emitter};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};
use std::sync::Arc;

mod commands;
mod privacy;
mod ai;
mod clipboard;
mod config;
mod regex;

use commands::AIState;
use config::ConfigManager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    env_logger::init();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_clipboard_manager::init())
        .manage(Arc::new(AIState::default()))
        .setup(|app| {
            log::info!("FlowPaste starting...");

            // Initialize Config Manager
            let config_manager = ConfigManager::init(app.handle())
                .expect("Failed to initialize config manager");
            app.manage(config_manager);

            let window = app.get_webview_window("main").unwrap();

            // Hide window initially in release mode
            #[cfg(not(debug_assertions))]
            {
                let _ = window.hide();
            }

            #[cfg(debug_assertions)]
            {
                window.open_devtools();
            }

            // Register global shortcut: Ctrl+Shift+V
            let shortcut = Shortcut::new(Some(Modifiers::CONTROL | Modifiers::SHIFT), Code::KeyV);
            let window_clone = window.clone();

            app.global_shortcut().on_shortcut(shortcut, move |_app, _shortcut, event| {
                // Only respond to key press, not release
                if event.state != ShortcutState::Pressed {
                    return;
                }
                log::info!("Global shortcut triggered");

                if window_clone.is_visible().unwrap_or(false) {
                    let _ = window_clone.hide();
                } else {
                    let _ = window_clone.show();
                    let _ = window_clone.set_focus();
                    // Emit event to frontend to refresh clipboard
                    let _ = window_clone.emit("panel:show", ());
                }
            })?;

            log::info!("Global shortcut registered: Ctrl+Shift+V");

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::greet,
            commands::scan_pii,
            commands::mask_pii,
            commands::restore_pii,
            commands::list_local_models,
            commands::check_ollama_health,
            commands::send_ai_request,
            commands::cancel_ai_request,
            commands::read_clipboard,
            commands::write_clipboard,
            commands::get_config,
            commands::set_config,
            commands::get_api_key,
            commands::set_api_key,
            commands::get_builtin_rules,
            commands::apply_rule,
            commands::apply_custom_rule,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

use tauri::Manager;
use std::sync::Arc;

mod commands;
mod privacy;
mod ai;
mod clipboard;
mod config;
mod regex;
mod hotkey;

use commands::AIState;
use config::ConfigManager;
use hotkey::HotkeyManager;

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

            // Initialize Hotkey Manager
            let hotkey_manager = HotkeyManager::new();
            app.manage(hotkey_manager);

            let window = app.get_webview_window("main")
                .expect("Main window not found - check tauri.conf.json");

            // Hide window initially in release mode
            #[cfg(not(debug_assertions))]
            {
                let _ = window.hide();
            }

            #[cfg(debug_assertions)]
            {
                window.open_devtools();
            }

            // Register default hotkey from config
            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                let config_mgr: tauri::State<ConfigManager> = app_handle.state();
                let hotkey_mgr: tauri::State<HotkeyManager> = app_handle.state();

                let hotkey_str = match config_mgr.get_config() {
                    Ok(cfg) => cfg.hotkey,
                    Err(_) => "Ctrl+Shift+V".to_string(),
                };

                if let Err(e) = hotkey_mgr.register_hotkey(&app_handle, &hotkey_str).await {
                    log::error!("Failed to register hotkey '{}': {}", hotkey_str, e);
                } else {
                    log::info!("Global shortcut registered: {}", hotkey_str);
                }
            });

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
            commands::detect_content_intent,
            commands::read_clipboard,
            commands::write_clipboard,
            commands::get_config,
            commands::set_config,
            commands::get_api_key,
            commands::set_api_key,
            commands::get_builtin_rules,
            commands::apply_rule,
            commands::apply_custom_rule,
            commands::register_hotkey,
            commands::unregister_hotkey,
            commands::is_hotkey_registered,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

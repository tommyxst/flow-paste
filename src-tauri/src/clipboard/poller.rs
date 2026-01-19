//! Clipboard History Poller
//!
//! Background task that monitors clipboard changes and records them to history.
//! Features:
//! - Adaptive polling interval (300-800ms)
//! - Deduplication via hash comparison
//! - Optional PII detection integration
//! - Resets hash state when history is cleared

use std::sync::Arc;
use std::time::Duration;
use tauri::{AppHandle, Emitter, Listener, Manager};
use tokio::sync::watch;

use crate::clipboard::{read_clipboard, ClipboardHistoryService, ClipboardKind};

pub struct PollerHandle {
    shutdown_tx: watch::Sender<bool>,
}

impl PollerHandle {
    #[allow(dead_code)]
    pub fn shutdown(&self) {
        let _ = self.shutdown_tx.send(true);
    }
}

pub fn start_clipboard_poller(app: AppHandle) -> PollerHandle {
    let (shutdown_tx, mut shutdown_rx) = watch::channel(false);

    tauri::async_runtime::spawn(async move {
        let mut interval_ms = super::history::MIN_POLL_INTERVAL_MS;
        let mut last_hash: Option<String> = None;

        // Listen for history clear events to reset last_hash
        let app_clone = app.clone();
        let (reset_tx, mut reset_rx) = tokio::sync::mpsc::channel::<()>(1);
        app_clone.listen("clipboard:history_cleared", move |_| {
            let _ = reset_tx.try_send(());
        });

        log::info!("Clipboard history poller started");

        loop {
            // Check for shutdown
            if *shutdown_rx.borrow() {
                log::info!("Clipboard history poller shutting down");
                break;
            }

            // Check for history clear signal (reset last_hash)
            if reset_rx.try_recv().is_ok() {
                log::debug!("Resetting poller last_hash due to history clear");
                last_hash = None;
            }

            // Wait for interval
            tokio::select! {
                _ = tokio::time::sleep(Duration::from_millis(interval_ms)) => {}
                _ = shutdown_rx.changed() => {
                    if *shutdown_rx.borrow() {
                        break;
                    }
                }
            }

            // Read clipboard
            let app_handle = app.clone();
            let read_result = tauri::async_runtime::spawn_blocking(move || {
                read_clipboard(&app_handle)
            })
            .await;

            let content = match read_result {
                Ok(Ok(content)) => content,
                Ok(Err(e)) => {
                    log::debug!("Clipboard read failed: {}", e);
                    interval_ms = super::history::next_poll_interval_ms(interval_ms, false);
                    continue;
                }
                Err(e) => {
                    log::error!("Clipboard read task panicked: {}", e);
                    interval_ms = super::history::next_poll_interval_ms(interval_ms, false);
                    continue;
                }
            };

            // Only process text content
            if content.kind != ClipboardKind::Text {
                interval_ms = super::history::next_poll_interval_ms(interval_ms, false);
                continue;
            }

            let text = match content.text {
                Some(ref t) if !t.trim().is_empty() => t.clone(),
                _ => {
                    interval_ms = super::history::next_poll_interval_ms(interval_ms, false);
                    continue;
                }
            };

            // Compute hash and check for change
            let text_hash = super::history::compute_text_hash(&text);

            if Some(&text_hash) == last_hash.as_ref() {
                // No change
                interval_ms = super::history::next_poll_interval_ms(interval_ms, false);
                continue;
            }

            // Try to record to history
            let service = match app.try_state::<Arc<ClipboardHistoryService>>() {
                Some(s) => s.inner().clone(),
                None => {
                    log::warn!("ClipboardHistoryService not available");
                    interval_ms = super::history::next_poll_interval_ms(interval_ms, false);
                    continue;
                }
            };

            // Check for PII
            let pii_detected = check_pii(&text);

            let text_clone = text.clone();
            let record_result = tauri::async_runtime::spawn_blocking(move || {
                service.record_text(&text_clone, "poll", pii_detected)
            })
            .await;

            match record_result {
                Ok(Ok(result)) => {
                    if result.inserted {
                        log::debug!("Recorded clipboard history item: {:?}", result.id);
                        last_hash = Some(text_hash);
                        interval_ms = super::history::next_poll_interval_ms(interval_ms, true);

                        // Emit event for frontend
                        let _ = app.emit("clipboard:history_changed", serde_json::json!({
                            "action": "insert",
                            "id": result.id
                        }));
                    } else {
                        log::debug!("Clipboard not recorded: {:?}", result.reason);
                        if result.reason != Some("debounced") {
                            last_hash = Some(text_hash);
                        }
                        interval_ms = super::history::next_poll_interval_ms(interval_ms, false);
                    }
                }
                Ok(Err(e)) => {
                    log::error!("Failed to record clipboard history: {}", e);
                    interval_ms = super::history::next_poll_interval_ms(interval_ms, false);
                }
                Err(e) => {
                    log::error!("Record task panicked: {}", e);
                    interval_ms = super::history::next_poll_interval_ms(interval_ms, false);
                }
            }
        }
    });

    PollerHandle { shutdown_tx }
}

fn check_pii(text: &str) -> bool {
    use crate::privacy::scan_pii;
    let result = scan_pii(text);
    result.has_pii
}

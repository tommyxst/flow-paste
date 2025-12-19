use crate::privacy::{self, MaskMapping, MaskResult, PIIScanResult};

#[tauri::command]
pub fn scan_pii(text: &str) -> PIIScanResult {
    privacy::scan_pii(text)
}

#[tauri::command]
pub fn mask_pii(text: &str) -> MaskResult {
    privacy::mask_pii(text)
}

#[tauri::command]
pub fn restore_pii(text: &str, mapping: MaskMapping) -> String {
    privacy::restore_pii(text, &mapping)
}

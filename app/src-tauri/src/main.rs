#[tauri::command]
fn scan_path(path: String) -> Result<Vec<jozin_core::Sidecar>, String> {
  jozin_core::api::scan_path(&path).map_err(|e| e.to_string())
}
fn main() {
  tauri::Builder::default()
    .invoke_handler(tauri::generate_handler![scan_path])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}

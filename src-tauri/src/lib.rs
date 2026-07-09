use tauri::Manager;
use tokio::sync::Mutex;

mod ipc;

pub struct AppState {
    pub bundle_path: Mutex<Option<String>>,
    pub index_engine: Mutex<Option<haven_index::IndexEngine>>,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(AppState {
            bundle_path: Mutex::new(None),
            index_engine: Mutex::new(None),
        })
        .invoke_handler(tauri::generate_handler![
            ipc::open_bundle,
            ipc::create_document,
            ipc::read_document,
            ipc::search_documents,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

pub mod api;
pub mod bluetooth;
pub mod elevated;
pub mod elevated_worker;
pub mod sync;

use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Initialize the global elevated worker (lazy — spawns on first use)
    elevated_worker::get_worker();

    let router = api::init();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            let _ = app
                .get_webview_window("main")
                .expect("no main window")
                .set_focus();
        }))
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_svelte::init())
        .invoke_handler(router.into_handler())
        .on_window_event(|_window, event| {
            if let tauri::WindowEvent::Destroyed = event {
                // Shut down the elevated worker when the app closes
                let worker = elevated_worker::get_worker();
                // Block on shutdown so the worker process is cleaned up before exit
                let rt = tokio::runtime::Handle::current();
                rt.block_on(async move {
                    worker.shutdown().await;
                });
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

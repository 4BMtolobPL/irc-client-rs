use crate::irc_client::IrcClient;
use tauri::{Emitter, Manager};

mod irc_client;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(
            tauri_plugin_log::Builder::new()
                .level(tauri_plugin_log::log::LevelFilter::Trace)
                .filter(|metadata| metadata.target().starts_with("irc_client_lib"))
                .build(),
        )
        .setup(|app| {
            let (tx, rx) = tokio::sync::mpsc::channel(100);
            app.manage(IrcClientState { irc_tx: tx.clone() });

            let app_handle = app.handle().clone();

            tauri::async_runtime::spawn(async move {
                match IrcClient::new().await {
                    Ok(client) => {
                        let _ = irc_client::run_irc_task(client, rx, app_handle).await;
                    }
                    Err(e) => {
                        eprintln!("Failed to connect to IRC client: {}", e);
                        let _ = app_handle.emit("irc:init_error", e.to_string());
                    }
                }
            });

            Ok(())
        })
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![irc_client::send_message])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

struct IrcClientState {
    irc_tx: tokio::sync::mpsc::Sender<irc_client::IrcCommand>,
}

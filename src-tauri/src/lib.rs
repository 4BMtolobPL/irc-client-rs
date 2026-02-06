use crate::irc_client::IrcClient;
use tauri::menu::*;
use tauri::{Emitter, Manager};

mod irc_client;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(
            // Logging
            tauri_plugin_log::Builder::new()
                .level(tauri_plugin_log::log::LevelFilter::Trace)
                .filter(|metadata| metadata.target().starts_with("irc_client_lib"))
                .build(),
        )
        .setup(|app| {
            let app_handle = app.handle();

            // Menu
            let file_menu = SubmenuBuilder::new(app, "File")
                // .submenu_icon(menu_image) // Optional: Add an icon to the submenu
                .text("open", "Open")
                .text("quit", "Quit")
                .build()?;

            let menu = MenuBuilder::new(app_handle).items(&[&file_menu]).build()?;

            app.set_menu(menu)?;

            /*// Update individual menu item text
            menu.get("status")
                .unwrap()
                .as_menuitem_unchecked()
                .set_text("Status: Ready")?;*/

            // IRC
            let (tx, rx) = tokio::sync::mpsc::channel(100);
            app.manage(IrcClientState { irc_tx: tx.clone() });

            let app_handle_irc = app.handle().clone();

            tauri::async_runtime::spawn(async move {
                match IrcClient::new().await {
                    Ok(client) => {
                        let _ = irc_client::run_irc_task(client, rx, app_handle_irc).await;
                    }
                    Err(e) => {
                        eprintln!("Failed to connect to IRC client: {}", e);
                        let _ = app_handle_irc.emit("irc:init_error", e.to_string());
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

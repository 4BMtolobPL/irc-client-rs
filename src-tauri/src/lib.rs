use crate::kirc::state::IRCClientState;
use tauri::menu::{MenuBuilder, SubmenuBuilder};
use tauri::Manager;
use tauri_plugin_log::log;

mod error;
mod kirc;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(
            // Logging
            tauri_plugin_log::Builder::new()
                .level(log::LevelFilter::Debug)
                .level_for("irc_client_lib", log::LevelFilter::Trace)
                .build(),
        )
        .setup(|app| {
            // Menu
            let file_menu = SubmenuBuilder::new(app, "File")
                // .submenu_icon(menu_image) // Optional: Add an icon to the submenu
                .text("open", "Open")
                .text("quit", "Quit")
                .build()?;

            let app_handle = app.handle();
            let menu = MenuBuilder::new(app_handle).items(&[&file_menu]).build()?;

            app.set_menu(menu)?;

            // kirc
            app.manage(IRCClientState::new());

            Ok(())
        })
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            kirc::commands::connect_server,
            kirc::commands::join_channel,
            kirc::commands::send_message
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

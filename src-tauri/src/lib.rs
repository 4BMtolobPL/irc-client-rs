use crate::kirc::state::IRCClientState;
use anyhow::Context;
use tauri::menu::{Menu, MenuBuilder, MenuEvent, MenuItem, SubmenuBuilder};
use tauri::tray::TrayIconBuilder;
use tauri::{AppHandle, Manager, Window, WindowEvent};
use tauri_plugin_log::log;
use tauri_plugin_log::log::warn;

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

            app.set_menu(menu).context("Can not initialize menu")?;

            // System tray
            let tray_quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let tray_show = MenuItem::with_id(app, "show", "Show", true, None::<&str>)?;
            let tray_menu = Menu::with_items(app, &[&tray_show, &tray_quit])?;
            let _tray = TrayIconBuilder::new()
                .menu(&tray_menu)
                .on_menu_event(on_menu_event)
                .show_menu_on_left_click(false)
                .icon(app.default_window_icon().unwrap().clone())
                .build(app)?;

            // kirc
            app.manage(IRCClientState::new());

            Ok(())
        })
        .on_window_event(on_window_event)
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            kirc::commands::connect_server,
            kirc::commands::join_channel,
            kirc::commands::send_message,
            kirc::commands::cancel_connect,
            kirc::commands::disconnect_server,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn on_window_event(window: &Window, event: &WindowEvent) {
    if let WindowEvent::CloseRequested { api, .. } = event {
        api.prevent_close(); // 실제 종료 막기
        let _ = window.hide(); // 창만 숨김
    }
}

fn on_menu_event(app_handle: &AppHandle, event: MenuEvent) {
    match event.id.as_ref() {
        "show" => {
            if let Some(window) = app_handle.get_webview_window("main") {
                let _ = window.show();
                let _ = window.unminimize();
                let _ = window.set_focus();
            }
        }
        "quit" => {
            let async_app_handle = app_handle.clone();

            tauri::async_runtime::spawn(async move {
                if let Some(state) = async_app_handle.try_state::<IRCClientState>() {
                    state.shutdown().await;
                }

                async_app_handle.exit(0);
            });
        }
        _ => {
            warn!("Unhandled event: {:?}", event.id);
        }
    }
}

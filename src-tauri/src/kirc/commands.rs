use crate::error::MyCustomError;
use crate::kirc::core::server_actor;
use crate::kirc::payloads::{ConnectServerPayload, ServerStatusPayload};
use crate::kirc::state::{IRCClientState, ServerRuntime};
use crate::kirc::types::{ServerCommand, ServerId, ServerStatus};
use anyhow::{anyhow, Context};
use tauri::{AppHandle, Emitter, State};
use tauri_plugin_log::log::info;

#[tauri::command]
pub(crate) async fn connect_server(
    payload: ConnectServerPayload,
    state: State<'_, IRCClientState>,
    app_handle: AppHandle,
) -> Result<(), MyCustomError> {
    if state.is_shutting_down() {
        return Err(MyCustomError::Anyhow(anyhow!(
            "Application is shutting down"
        )));
    }

    let server_id = payload.server_id.clone();
    let mut servers = state.servers.lock().unwrap();

    match servers.get(&server_id) {
        Some(
            ServerRuntime::Connecting { .. }
            | ServerRuntime::Registering { .. }
            | ServerRuntime::Connected { .. },
        ) => {
            return Err(MyCustomError::IRCServer(
                "Already connecting or connected".into(),
            ));
        }
        _ => {}
    }

    let handle = tokio::spawn(server_actor(server_id.clone(), payload, app_handle.clone()));

    servers.insert(server_id.clone(), ServerRuntime::Connecting { handle });

    app_handle
        .emit(
            "kirc:server_status",
            ServerStatusPayload {
                server_id: server_id.to_string(),
                status: ServerStatus::Connecting,
            },
        )
        .context("Failed to emit kirc:server_connecting")?;

    Ok(())
}

#[tauri::command]
pub(crate) fn join_channel(
    server_id: ServerId,
    channel: String,
    state: State<IRCClientState>,
) -> Result<(), MyCustomError> {
    info!("Tauri command: join channel invoked, server_id: {server_id}, channel: {channel}");

    let servers = state.servers.lock().expect("Servers lock poisoned");
    let server = servers.get(&server_id).context("Can't find server")?;

    if let ServerRuntime::Connected { tx, .. } = server {
        tx.send(ServerCommand::Join(channel))
            .context("Failed to send join command")?;
    }

    Ok(())
}

#[tauri::command]
pub(crate) fn send_message(
    server_id: ServerId,
    target: String,
    message: String,
    state: State<IRCClientState>,
) -> Result<(), MyCustomError> {
    info!("Tauri command: send message invoked, server_id: {server_id}, target: {target}, message: {message}");

    let servers = state.servers.lock().expect("Servers lock poisoned");
    let server = servers.get(&server_id).context("Can't find server")?;

    if let ServerRuntime::Connected { tx, .. } = server {
        tx.send(ServerCommand::Privmsg { target, message })
            .context("Failed to send privmsg")?;
    }

    Ok(())
}

#[tauri::command]
pub(crate) fn cancel_connect(
    server_id: ServerId,
    state: State<IRCClientState>,
    app_handle: AppHandle,
) -> Result<(), MyCustomError> {
    info!("Tauri command: cancel connect invoked, server_id: {server_id}");
    let mut servers = state.servers.lock().expect("Servers lock poisoned");

    if let Some(ServerRuntime::Connecting { handle }) = servers.remove(&server_id) {
        handle.abort();

        servers.insert(server_id.clone(), ServerRuntime::Disconnected);

        app_handle
            .emit(
                "kirc:server_status",
                ServerStatusPayload {
                    server_id: server_id.to_string(),
                    status: ServerStatus::Failed,
                },
            )
            .context("Failed to emit kirc:server_status")?;
    }

    Ok(())
}

#[tauri::command]
pub(crate) fn disconnect_server(
    server_id: String,
    state: State<IRCClientState>,
) -> Result<(), MyCustomError> {
    let mut servers = state.servers.lock().expect("Servers lock poisoned");

    if let Some(runtime) = servers.remove(&server_id) {
        match runtime {
            ServerRuntime::Registering { tx, handle } | ServerRuntime::Connected { tx, handle } => {
                let _ = tx.send(ServerCommand::Quit);
                servers.insert(server_id, ServerRuntime::Disconnecting { handle });
            }
            other => {
                servers.insert(server_id, other);
            }
        }
    }

    Ok(())
}

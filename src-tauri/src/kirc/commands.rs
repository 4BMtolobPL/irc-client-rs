use crate::error::MyCustomError;
use crate::kirc::core::start_event_loop;
use crate::kirc::payloads::ConnectServerPayload;
use crate::kirc::state::{IRCClientState, ServerHandle};
use crate::kirc::types::{ServerCommand, ServerId, ServerStatus};
use anyhow::Context;
use irc::client::data::Config;
use irc::client::Client;
use tauri::{AppHandle, Emitter, State};
use tauri_plugin_log::log::trace;

#[tauri::command]
pub(crate) async fn connect_server(
    payload: ConnectServerPayload,
    state: State<'_, IRCClientState>,
    app_handle: AppHandle,
) -> Result<(), MyCustomError> {
    trace!("connect server: {:?}", payload);

    {
        let mut statuses = state.statuses.lock().expect("Statuses lock poisoned");

        if matches!(
            statuses.get(&payload.server_id),
            Some(ServerStatus::Connecting | ServerStatus::Connected)
        ) {
            // TODO: 이미 연결되어 있다면 그 서버를 꺼내서 리턴 해 줄 수도?
            return Err(MyCustomError::IRCServer(
                "IRC server already connected".to_string(),
            ));
        }
        statuses.insert(payload.server_id.clone(), ServerStatus::Connecting);
    }

    let config = Config {
        server: Some(payload.host),
        port: Some(payload.port),
        use_tls: Some(payload.tls),
        nickname: Some(payload.nickname),
        // channels: vec!["#test".to_string()],
        ..Config::default()
    };
    let client = Client::from_config(config)
        .await
        .context("Failed to create client")?;
    client.identify().context("Failed to identify client")?;

    // https://docs.rs/tokio/1.49.0/tokio/sync/mpsc/fn.unbounded_channel.html
    // 메모리 부족 문제가 생길 수 있음.
    // -> tokio::sync::mpsc::channel(buffer)로 변경 할 수 있음
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel();

    // client를 내부 루프로 소유권 이동
    start_event_loop(payload.server_id.clone(), client, rx, app_handle.clone());

    {
        state
            .statuses
            .lock()
            .expect("Statuses lock poisoned")
            .insert(payload.server_id.clone(), ServerStatus::Connected);
    }
    {
        state
            .servers
            .lock()
            .expect("Servers lock poisoned")
            .insert(payload.server_id.clone(), ServerHandle { tx });
    }

    app_handle
        .emit("kirc:server_connected", payload.server_id)
        .context("Failed to emit kirc:server_connected")?;

    Ok(())
}

#[tauri::command]
pub(crate) fn join_channel(
    server_id: ServerId,
    channel: String,
    state: State<IRCClientState>,
) -> Result<(), MyCustomError> {
    let servers = state.servers.lock().expect("Servers lock poisoned");
    let server = servers.get(&server_id).context("Can't find server")?;

    server
        .tx
        .send(ServerCommand::Join(channel))
        .context("Failed to send join command")?;

    Ok(())
}

#[tauri::command]
pub(crate) fn send_message(
    server_id: ServerId,
    target: String,
    message: String,
    state: State<IRCClientState>,
) -> Result<(), MyCustomError> {
    let servers = state.servers.lock().expect("Servers lock poisoned");
    let server = servers.get(&server_id).context("Can't find server")?;

    server
        .tx
        .send(ServerCommand::Privmsg { target, message })
        .context("Failed to send privmsg")?;

    Ok(())
}

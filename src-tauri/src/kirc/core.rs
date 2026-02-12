use crate::kirc::emits::emit_disconnected;
use crate::kirc::payloads::{IrcMessagePayload, ServerStatusPayload, SystemMessagePayload};
use crate::kirc::state::IRCClientState;
use crate::kirc::types::{ServerCommand, ServerId, ServerStatus};
use anyhow::Context;
use futures::prelude::*;
use irc::client::prelude::*;
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_log::log::{debug, trace};
use tokio::sync::mpsc::UnboundedReceiver;

pub(super) fn start_event_loop(
    server_id: ServerId,
    mut client: Client,
    mut rx: UnboundedReceiver<ServerCommand>,
    app_handle: AppHandle,
) {
    trace!("Starting event loop");

    tokio::spawn(async move {
        let state = app_handle.state::<IRCClientState>();

        let mut stream = match client.stream() {
            Ok(s) => s,
            Err(_) => {
                debug!("Can't open stream");
                emit_disconnected(&server_id, &app_handle, &state);
                return;
            }
        };

        loop {
            trace!("Start stream loop");
            tokio::select! {
                // IRC → Frontend
                Some(result) = stream.next() => {
                    match result {
                        Ok(message) => {
                            handle_message(&server_id, message, &app_handle).expect("Failed to handle message");
                        },
                        _ => {
                            trace!("No message");
                        },
                    }
                },
                // Frontend → IRC
                Some(cmd) = rx.recv() => {
                    match cmd {
                        ServerCommand::Join(ch) => {
                            trace!("Command Join");
                            let _ = client.send_join(&ch);
                        }
                        ServerCommand::Privmsg { target, message } => {
                            trace!("Command Privmsg");
                            let _ = client.send_privmsg(&target, &message);
                        }
                        ServerCommand::Quit => {
                            trace!("Command Quit");
                            let _ = client.send_quit("bye");
                            break;
                        }
                    }
                }
            }
        }

        trace!("End stream loop");
        emit_disconnected(&server_id, &app_handle, &state);
    });
}

fn handle_message(server_id: &str, msg: Message, app_handle: &AppHandle) -> anyhow::Result<()> {
    match msg.command {
        Command::PRIVMSG(target, content) => {
            let payload = IrcMessagePayload {
                server_id: server_id.to_string(),
                channel: target,
                from: get_sender(msg.prefix.unwrap()),
                message: content,
                timestamp: chrono::Utc::now().timestamp_millis(),
            };

            app_handle
                .emit("kirc:message", payload)
                .context("Failed to send message")?;
        }
        Command::Response(Response::RPL_WELCOME, _) => {
            trace!("Response WELCOME | server_id: {}", server_id);
            {
                let state = app_handle.state::<IRCClientState>();
                let mut statuses = state.statuses.lock().expect("Failed to lock statuses");
                statuses.insert(server_id.to_string(), ServerStatus::Connected);
            }

            app_handle
                .emit(
                    "kirc:server_status",
                    ServerStatusPayload {
                        server_id: server_id.to_string(),
                        status: ServerStatus::Connected,
                    },
                )
                .context("Failed to emit server status")?;

            // Optional: Alert system message
            app_handle
                .emit(
                    "kirc:system_message",
                    SystemMessagePayload {
                        server_id: server_id.to_string(),
                        message: "서버에 연결되었습니다.".to_string(),
                    },
                )
                .context("Failed to emit system message")?;
        }
        _ => {
            // TODO: Command 다른것도 추가하기
        }
    }

    Ok(())
}

fn get_sender(prefix: Prefix) -> String {
    match prefix {
        Prefix::ServerName(servername) => servername,
        Prefix::Nickname(nickname, _, _) => nickname,
    }
}

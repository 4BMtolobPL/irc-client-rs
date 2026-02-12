use crate::kirc::emits::emit_disconnected;
use crate::kirc::payloads::{ServerStatusPayload, SystemMessagePayload, UiEventPayload};
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

fn handle_message(server_id: &str, message: Message, app_handle: &AppHandle) -> anyhow::Result<()> {
    match message.command {
        Command::PRIVMSG(target, content) => {
            if let Some(prefix) = message.prefix {
                let payload = UiEventPayload::UserMessage {
                    server_id: server_id.to_string(),
                    channel: target,
                    nick: get_sender(prefix),
                    content,
                    timestamp: chrono::Utc::now().timestamp_millis() as u64,
                };

                app_handle
                    .emit("kirc:event", payload)
                    .context("Failed to send message")?;
            } else {
                debug!("PRIVMSG message missing prefix");
            }
        }
        Command::JOIN(chanlist, _chankey, _real_name) => {
            if let Some(prefix) = message.prefix {
                let payload = UiEventPayload::Join {
                    server_id: server_id.to_string(),
                    channel: chanlist,
                    nick: get_sender(prefix),
                };

                app_handle
                    .emit("kirc:event", payload)
                    .context("Failed to send message")?;
            } else {
                debug!("JOIN message missing prefix");
            }
        }
        Command::PART(chanlist, comment) => {
            if let Some(prefix) = message.prefix {
                let payload = UiEventPayload::Part {
                    server_id: server_id.to_string(),
                    channel: chanlist,
                    nick: get_sender(prefix),
                    reason: comment,
                };

                app_handle
                    .emit("kirc:event", payload)
                    .context("Failed to send message")?;
            } else {
                debug!("PART message missing prefix");
            }
        }
        Command::QUIT(comment) => {
            if let Some(prefix) = message.prefix {
                let payload = UiEventPayload::Quit {
                    server_id: server_id.to_string(),
                    nick: get_sender(prefix),
                    reason: comment,
                };

                app_handle
                    .emit("kirc:event", payload)
                    .context("Failed to send message")?;
            } else {
                debug!("QUIT message missing prefix");
            }
        }
        Command::NICK(nickname) => {
            if let Some(prefix) = message.prefix {
                let payload = UiEventPayload::Nick {
                    server_id: server_id.to_string(),
                    old_nick: get_sender(prefix),
                    new_nick: nickname,
                };

                app_handle
                    .emit("kirc:event", payload)
                    .context("Failed to send message")?;
            } else {
                debug!("NICK message missing prefix");
            }
        }
        Command::TOPIC(channel, topic) => {
            let payload = UiEventPayload::Topic {
                server_id: server_id.to_string(),
                channel,
                topic,
            };

            app_handle
                .emit("kirc:event", payload)
                .context("Failed to send message")?;
        }
        Command::ERROR(message) => {
            let payload = UiEventPayload::Error {
                server_id: server_id.to_string(),
                message,
            };

            app_handle
                .emit("kirc:event", payload)
                .context("Failed to send message")?;
        }
        Command::Response(Response::RPL_WELCOME, _) => {
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

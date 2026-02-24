use crate::kirc::payloads::*;
use crate::kirc::state::{IRCClientState, ServerRuntime};
use crate::kirc::types::{ServerCommand, ServerStatus};
use anyhow::Context;
use futures::prelude::*;
use irc::client::prelude::*;
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_log::log::{error, trace};

pub(super) async fn server_actor(
    server_id: String,
    payload: ConnectServerPayload,
    app_handle: AppHandle,
) {
    // actor에선 error를 ?로 전파하지 않고, 소비/로깅만 하거나 이벤트로 전파

    let config = Config {
        server: Some(payload.host),
        port: Some(payload.port),
        use_tls: Some(payload.tls),
        nickname: Some(payload.nickname),
        ..Config::default()
    };

    let mut client = match Client::from_config(config).await {
        Ok(c) => c,
        Err(e) => {
            fail_state(&server_id, &app_handle, e.to_string());
            return;
        }
    };

    if let Err(e) = client.identify() {
        fail_state(&server_id, &app_handle, e.to_string());
        return;
    }

    let mut stream = match client.stream() {
        Ok(s) => s,
        Err(e) => {
            fail_state(&server_id, &app_handle, e.to_string());
            return;
        }
    };

    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();

    {
        let state = app_handle.state::<IRCClientState>();
        let mut servers = state.servers.lock().unwrap();
        if let Some(ServerRuntime::Connecting { handle }) = servers.remove(&server_id) {
            trace!("server_actor status connecting to registering");
            servers.insert(
                server_id.clone(),
                ServerRuntime::Registering {
                    tx: tx.clone(),
                    handle,
                },
            );
        }
    }

    let _ = app_handle.emit(
        "kirc:server_status",
        ServerStatusPayload {
            server_id: server_id.to_string(),
            status: ServerStatus::Registering,
        },
    );

    loop {
        tokio::select! {
            Some(result) = stream.next() => {
                match result {
                    Ok(message) => {
                        let _ = handle_message(&server_id, message, &app_handle);
                    }
                    Err(_) => break,
                }
            }
            Some(cmd) = rx.recv() => {
                trace!("rx recieve: {cmd}");
                match cmd {
                    ServerCommand::Join(ch) => {
                        if let Err(e) = client.send_join(&ch) {
                            error!("Failed to send join message: {e}");
                        }
                    }
                    ServerCommand::Privmsg { target, message } => {
                        if let Err(e) = client.send_privmsg(&target, &message) {
                            error!("Failed to send privmsg: {e}");
                        }

                        match Message::with_tags(None, Some(client.current_nickname()), "PRIVMSG", vec![&target, &message]) {
                                Ok(msg) => {
                                    trace!("Create echo: {:?}", msg);
                                    handle_message(&server_id, msg, &app_handle).expect("Failed to handle message");
                                }
                                Err(_) => {
                                    error!("Failed to create echo message");
                                }
                            }
                    }
                    ServerCommand::Quit => {
                        if let Err(e) = client.send_quit("bye") {
                            error!("Failed to send quit message: {e}");
                        }
                        break;
                    }
                }
            }
        }
    }

    {
        let state = app_handle.state::<IRCClientState>();
        let mut servers = state.servers.lock().unwrap();
        servers.insert(server_id.clone(), ServerRuntime::Disconnected);
    }

    let _ = app_handle.emit(
        "kirc:server_status",
        ServerStatusPayload {
            server_id: server_id.to_string(),
            status: ServerStatus::Disconnected,
        },
    );
}

fn fail_state(server_id: &str, app_handle: &AppHandle, message: String) {
    let state = app_handle.state::<IRCClientState>();
    let mut servers = state.servers.lock().unwrap();

    servers.insert(
        server_id.to_string(),
        ServerRuntime::Failed { error: message },
    );

    let _ = app_handle.emit(
        "kirc:server_status",
        ServerStatusPayload {
            server_id: server_id.to_string(),
            status: ServerStatus::Failed,
        },
    );
}

fn handle_message(server_id: &str, message: Message, app_handle: &AppHandle) -> anyhow::Result<()> {
    let source_nickname = message.source_nickname().unwrap_or_else(|| "").to_string();

    match message.command {
        Command::PRIVMSG(target, content) => {
            trace!("PRIVMSG | from: {source_nickname}, target: {target}, content: {content}");

            let payload = UiEventPayload::UserMessage {
                server_id: server_id.to_string(),
                channel: target,
                nick: source_nickname.to_string(),
                content,
                timestamp: chrono::Utc::now().timestamp_millis() as u64,
            };

            app_handle
                .emit("kirc:event", payload)
                .context("Failed to send message")?;
        }
        Command::JOIN(chanlist, _chankey, _real_name) => {
            let payload = UiEventPayload::Join {
                server_id: server_id.to_string(),
                channel: chanlist,
                nick: source_nickname.to_string(),
            };

            app_handle
                .emit("kirc:event", payload)
                .context("Failed to send message")?;
        }
        Command::PART(chanlist, comment) => {
            let payload = UiEventPayload::Part {
                server_id: server_id.to_string(),
                channel: chanlist,
                nick: source_nickname.to_string(),
                reason: comment,
            };

            app_handle
                .emit("kirc:event", payload)
                .context("Failed to send message")?;
        }
        Command::QUIT(comment) => {
            let payload = UiEventPayload::Quit {
                server_id: server_id.to_string(),
                nick: source_nickname.to_string(),
                reason: comment,
            };

            app_handle
                .emit("kirc:event", payload)
                .context("Failed to send message")?;
        }
        Command::NICK(nickname) => {
            let payload = UiEventPayload::Nick {
                server_id: server_id.to_string(),
                old_nick: source_nickname.to_string(),
                new_nick: nickname,
            };

            app_handle
                .emit("kirc:event", payload)
                .context("Failed to send message")?;
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
            trace!("handle_message RPL_WELCOME");
            {
                let state = app_handle.state::<IRCClientState>();
                let mut servers = state.servers.lock().expect("Failed to lock servers");
                if let Some(ServerRuntime::Registering { tx, handle }) = servers.remove(server_id) {
                    servers.insert(
                        server_id.to_string(),
                        ServerRuntime::Connected { tx, handle },
                    );
                }
            }

            app_handle
                .emit(
                    "kirc:server_status",
                    ServerStatusPayload {
                        server_id: server_id.to_string(),
                        status: ServerStatus::Connected,
                    },
                )
                .context("Failed to emit kirc:server_status")?;

            // Optional: Alert system message
            app_handle
                .emit(
                    "kirc:system_message",
                    SystemMessagePayload {
                        server_id: server_id.to_string(),
                        message: "서버에 연결되었습니다.".to_string(),
                    },
                )
                .context("Failed to emit kirc:system_message")?;
        }
        _ => {
            // TODO: Command 다른것도 추가하기
        }
    }

    Ok(())
}

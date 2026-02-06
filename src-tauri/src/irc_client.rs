use crate::IrcClientState;
use futures::StreamExt;
use irc::client::prelude::*;
use serde::Serialize;
use tauri::Emitter;
use tauri_plugin_log::log::{debug, error, trace};

#[derive(Debug)]
pub(crate) enum IrcCommand {
    Send { channel: String, message: String },
    JoinChannel(String),
    Quit,
}

pub(crate) struct IrcClient {
    client: Client,
}

#[derive(Clone, Serialize)]
struct IrcMessage {
    from: String,
    to: String,
    message: String,
}

impl IrcClient {
    pub(crate) async fn new(/*handle: tauri::AppHandle*/) -> Result<Self, anyhow::Error> {
        // let config_dir = handle.path().app_config_dir(); TODO: 설정 폴더 이동
        let config = Config::load("config.toml")?;
        let client = Client::from_config(config).await?;
        client.identify()?;

        Ok(Self { client })
    }
}

/// IRC Client는 이 함수만 소유
pub(crate) async fn run_irc_task(
    mut irc_client: IrcClient,
    mut rx: tokio::sync::mpsc::Receiver<IrcCommand>,
    app_handle: tauri::AppHandle,
) -> Result<(), anyhow::Error> {
    let mut stream = irc_client.client.stream()?;

    loop {
        tokio::select! {
            Some(cmd) = rx.recv() => {
                trace!("received command: {:?}", cmd);

                match cmd {
                    IrcCommand::Send{ channel, message } => {
                        let _ = irc_client.client.send(Command::PRIVMSG(channel, message));
                    }
                    IrcCommand::JoinChannel(_) => {}
                    IrcCommand::Quit => {}
                }


            }
            Some(result) = stream.next() => {
                match result {
                    Ok(message) => {
                        match message.command {
                            Command::PRIVMSG(msg_target, msg) => {
                                let from = {
                                    if let Some(x) = message.prefix {
                                        match x {
                                            Prefix::ServerName(servername) => servername,
                                            Prefix::Nickname(nickname, _, _) => nickname
                                        }
                                    } else {
                                        "".to_string()
                                    }
                                };

                                let _ = app_handle.emit("irc:message", IrcMessage { from, to: msg_target, message: msg });
                            }
                            _ => {
                                debug!("{:?}", message);
                            }
                        }
                    }
                    Err(e) => {
                        error!("IRC client error: {}", e);
                    }
                }
            }
        }
    }
}

#[tauri::command]
pub(crate) async fn send_message(
    state: tauri::State<'_, IrcClientState>,
    app_handle: tauri::AppHandle,
    channel: String,
    message: String,
) -> Result<(), String> {
    trace!("Sending message: {}", message);

    state
        .irc_tx
        .send(IrcCommand::Send {
            channel: channel.clone(),
            message: message.clone(),
        })
        .await
        .map_err(|e| e.to_string())?;

    app_handle
        .emit(
            "irc:message",
            IrcMessage {
                from: "me".to_string(),
                to: channel,
                message,
            },
        )
        .map_err(|e| e.to_string())
    // TODO: 자체 에러 타입 구현
    /*
    #[derive(Debug, Serialize)]
    #[serde(tag = "type", content = "data")]
    enum IrcErrorDto {
        NotConnected,
        AuthFailed,
        IoError(String),
        Unknown(String),
    }
    이런느낌으루다가.
    아니면 thiserror 같이 써도 될듯.
    */
}

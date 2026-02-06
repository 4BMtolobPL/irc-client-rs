use crate::IrcClientState;
use futures::StreamExt;
use irc::client::prelude::*;
use tauri::Emitter;

pub(crate) enum IrcCommand {
    Send { channel: String, message: String },
    JoinChannel(String),
    Quit,
}

pub(crate) struct IrcClient {
    client: Client,
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

    while let Some(message) = stream.next().await.transpose()? {
        match message.command {
            Command::PRIVMSG(msg_target, msg) => {
                let _ = app_handle.emit("irc:message", (msg_target, msg));
            }
            _ => {}
        }
    }

    Ok(())
}

#[tauri::command]
pub(crate) async fn send_message(
    state: tauri::State<'_, IrcClientState>,
    channel: String,
    message: String,
) -> Result<(), String> {
    trace!("Sending message: {}", message);

    state
        .irc_tx
        .send(IrcCommand::Send { channel, message })
        .await
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

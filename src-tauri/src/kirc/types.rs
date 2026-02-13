use serde::Serialize;

pub(super) type ServerId = String;

#[derive(Serialize, Clone, Default)]
pub(super) enum ServerStatus {
    Connecting,
    Connected,
    #[default]
    Disconnected,
}

pub(crate) enum ServerCommand {
    Join(String),
    Privmsg { target: String, message: String },
    Quit,
}

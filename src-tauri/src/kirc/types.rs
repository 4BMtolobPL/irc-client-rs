use serde::Serialize;

pub(super) type ServerId = String;

#[derive(Serialize, Clone)]
pub(super) enum ServerStatus {
    Connecting,
    Connected,
    Disconnected,
}

impl Default for ServerStatus {
    fn default() -> Self {
        Self::Disconnected
    }
}

pub(crate) enum ServerCommand {
    Join(String),
    Privmsg { target: String, message: String },
    Quit,
}

use crate::kirc::types::{ServerId, ServerStatus};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug)]
pub(crate) struct ConnectServerPayload {
    pub(super) server_id: ServerId,
    pub(super) host: String,
    pub(super) port: u16,
    pub(super) tls: bool,
    pub(super) nickname: String,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub(super) struct ServerStatusPayload {
    pub(super) server_id: ServerId,
    pub(super) status: ServerStatus,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub(super) struct SystemMessagePayload {
    pub(super) server_id: ServerId,
    pub(super) message: String,
}

#[derive(Debug, Serialize, Clone)]
#[serde(tag = "type")]
pub(super) enum UiEventPayload {
    UserMessage {
        server_id: String,
        channel: String,
        nick: String,
        content: String,
        timestamp: u64,
    },
    Join {
        server_id: String,
        channel: String,
        nick: String,
    },
    Part {
        server_id: String,
        channel: String,
        nick: String,
        reason: Option<String>,
    },
    Quit {
        server_id: String,
        nick: String,
        reason: Option<String>,
    },
    Nick {
        server_id: String,
        old_nick: String,
        new_nick: String,
    },
    Topic {
        server_id: String,
        channel: String,
        topic: Option<String>,
    },
    Error {
        server_id: String,
        message: String,
    },
}

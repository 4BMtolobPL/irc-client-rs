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

#[derive(Serialize)]
#[derive(Clone)]
#[serde(rename_all = "camelCase")]
pub(super) struct ServerStatusPayload {
    pub(super) server_id: ServerId,
    pub(super) status: ServerStatus,
}

#[derive(Serialize)]
#[derive(Clone)]
#[serde(rename_all = "camelCase")]
pub(super) struct SystemMessagePayload {
    pub(super) server_id: ServerId,
    pub(super) message: String,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct IrcMessagePayload {
    pub(super) server_id: ServerId,
    pub(super) channel: String,
    pub(super) from: String,
    pub(super) message: String,
    pub(super) timestamp: i64,
}
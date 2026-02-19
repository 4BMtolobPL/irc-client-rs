use crate::kirc::types::{ServerCommand, ServerId, ServerStatus};
use std::collections::HashMap;
use std::sync::Mutex;
use tokio::sync::mpsc::UnboundedSender;

pub(super) struct ServerHandle {
    pub(super) tx: UnboundedSender<ServerCommand>,
}

pub(crate) struct IRCClientState {
    pub(super) servers: Mutex<HashMap<ServerId, ServerHandle>>,
    pub(super) statuses: Mutex<HashMap<ServerId, ServerStatus>>,
}

impl IRCClientState {
    pub(crate) fn new() -> Self {
        Self {
            servers: Mutex::new(HashMap::new()),
            statuses: Mutex::new(HashMap::new()),
        }
    }

    pub(crate) fn shutdown(&self) {
        for server in self
            .servers
            .lock()
            .expect("IRCClientState shutdown servers mutex poisoned")
            .values()
        {
            let _ = server.tx.send(ServerCommand::Quit);
        }
    }
}

use crate::kirc::types::{ChannelId, ServerCommand, ServerId};
use anyhow::anyhow;
use std::cmp::PartialEq;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::sync::mpsc::UnboundedSender;
use tokio::task::JoinHandle;
use tokio::time::timeout;

#[derive(Default)]
pub(super) enum ServerRuntime {
    #[default]
    Disconnected,
    Connecting {
        handle: JoinHandle<()>,
    },
    Registering {
        tx: UnboundedSender<ServerCommand>,
        handle: JoinHandle<()>,
    },
    Connected {
        tx: UnboundedSender<ServerCommand>,
        handle: JoinHandle<()>,
    },
    Disconnecting {
        handle: JoinHandle<()>,
    },
    Failed {
        error: String,
    },
}

impl ServerRuntime {
    /*fn status(&self) -> ServerStatus {
        match self {
            ServerRuntime::Disconnected => ServerStatus::Disconnected,
            ServerRuntime::Connecting { .. } => ServerStatus::Connecting,
            ServerRuntime::Registering { .. } => ServerStatus::Registering,
            ServerRuntime::Connected { .. } => ServerStatus::Connected,
            ServerRuntime::Disconnecting { .. } => ServerStatus::Disconnecting,
            ServerRuntime::Failed { .. } => ServerStatus::Failed,
        }
    }*/

    async fn graceful_shutdown(self) {
        const TIMEOUT: Duration = Duration::from_secs(5);

        match self {
            ServerRuntime::Connected { tx, handle } | ServerRuntime::Registering { tx, handle } => {
                // 1. QUIT 전송
                let _ = tx.send(ServerCommand::Quit);

                // 2. 정상 종료 대기 (timeout optional)
                let _ = timeout(TIMEOUT, handle).await;
            }

            ServerRuntime::Connecting { handle } => {
                handle.abort();
                let _ = handle.await;
            }

            ServerRuntime::Disconnecting { handle } => {
                let _ = timeout(TIMEOUT, handle).await;
            }

            ServerRuntime::Disconnected | ServerRuntime::Failed { .. } => {
                // nothing
            }
        }
    }
}

#[derive(PartialEq)]
pub(crate) enum AppState {
    Running,
    ShuttingDown,
    Terminated,
}

impl AppState {
    fn as_u8(&self) -> u8 {
        match self {
            AppState::Running => 0,
            AppState::ShuttingDown => 1,
            AppState::Terminated => 2,
        }
    }

    fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(Self::Running),
            1 => Some(Self::ShuttingDown),
            2 => Some(Self::Terminated),
            _ => None,
        }
    }
}

#[derive(Default)]
pub(super) struct ChannelState {
    pub(super) locked: bool,
}

pub(super) struct ServerState {
    runtime: Mutex<ServerRuntime>,
    channels: Mutex<HashMap<ChannelId, ChannelState>>,
}

impl ServerState {
    pub(super) fn new(runtime: ServerRuntime) -> Self {
        Self {
            runtime: Mutex::new(runtime),
            channels: Mutex::new(HashMap::new()),
        }
    }

    /*pub(super) fn status(&self) -> ServerStatus {
        self.runtime.lock().unwrap().status()
    }*/

    pub(super) fn is_active(&self) -> bool {
        matches!(
            &*self.runtime.lock().unwrap(),
            ServerRuntime::Connecting { .. }
                | ServerRuntime::Registering { .. }
                | ServerRuntime::Connected { .. }
        )
    }

    pub(super) fn is_channel_locked(&self, channel: &str) -> bool {
        self.channels
            .lock()
            .unwrap()
            .get(channel)
            .map(|s| s.locked)
            .unwrap_or(false)
    }

    pub(super) fn set_channel_locked(&self, channel: &str, locked: bool) {
        self.channels
            .lock()
            .unwrap()
            .entry(channel.to_string())
            .or_default()
            .locked = locked;
    }

    pub(super) fn send_command(&self, cmd: ServerCommand) -> anyhow::Result<()> {
        let guard = self.runtime.lock().unwrap();
        match &*guard {
            ServerRuntime::Connected { tx, .. } | ServerRuntime::Registering { tx, .. } => {
                tx.send(cmd).map_err(|e| anyhow!("Failed to send: {}", e))
            }
            _ => Err(anyhow!("Server not connected")),
        }
    }

    pub(super) fn transition_to_registering(&self, tx: UnboundedSender<ServerCommand>) {
        let mut guard = self.runtime.lock().unwrap();
        if let ServerRuntime::Connecting { handle } = std::mem::take(&mut *guard) {
            *guard = ServerRuntime::Registering { tx, handle };
        }
    }

    pub(super) fn transition_to_connected(&self) {
        let mut guard = self.runtime.lock().unwrap();
        if let ServerRuntime::Registering { tx, handle } = std::mem::take(&mut *guard) {
            *guard = ServerRuntime::Connected { tx, handle };
        }
    }

    pub(super) fn transition_to_disconnected(&self) {
        *self.runtime.lock().unwrap() = ServerRuntime::Disconnected;
    }

    pub(super) fn transition_to_failed(&self, error: String) {
        *self.runtime.lock().unwrap() = ServerRuntime::Failed { error };
    }

    pub(super) fn disconnect(&self) {
        let mut guard = self.runtime.lock().unwrap();
        match std::mem::take(&mut *guard) {
            ServerRuntime::Registering { tx, handle } | ServerRuntime::Connected { tx, handle } => {
                let _ = tx.send(ServerCommand::Quit);
                *guard = ServerRuntime::Disconnecting { handle };
            }
            other => {
                *guard = other;
            }
        }
    }

    pub(super) fn abort_connecting(&self) -> bool {
        let mut guard = self.runtime.lock().unwrap();
        if let ServerRuntime::Connecting { handle } = std::mem::take(&mut *guard) {
            handle.abort();
            *guard = ServerRuntime::Disconnected;
            true
        } else {
            false
        }
    }

    pub(super) fn take_runtime(&self) -> ServerRuntime {
        std::mem::take(&mut *self.runtime.lock().unwrap())
    }
}

pub(crate) struct IRCClientState {
    servers: Mutex<HashMap<ServerId, Arc<ServerState>>>,
    app_state: AtomicU8,
}

impl IRCClientState {
    pub(crate) fn new() -> Self {
        Self {
            servers: Mutex::new(HashMap::new()),
            app_state: AtomicU8::new(AppState::Running.as_u8()),
        }
    }

    pub(super) fn get_server(&self, server_id: ServerId) -> Option<Arc<ServerState>> {
        self.servers.lock().unwrap().get(&server_id).cloned()
    }

    pub(super) fn add_server(&self, server_id: ServerId, state: ServerState) {
        self.servers
            .lock()
            .unwrap()
            .insert(server_id, Arc::new(state));
    }

    pub(super) fn ensure_server(&self, server_id: ServerId) -> Arc<ServerState> {
        let mut servers = self.servers.lock().unwrap();
        servers
            .entry(server_id)
            .or_insert_with(|| Arc::new(ServerState::new(ServerRuntime::Disconnected)))
            .clone()
    }

    fn app_state(&self) -> Option<AppState> {
        AppState::from_u8(self.app_state.load(Ordering::Acquire))
    }

    fn set_app_state(&self, state: AppState) {
        self.app_state.store(state.as_u8(), Ordering::Release);
    }

    pub(super) fn is_shutting_down(&self) -> bool {
        if let Some(state) = self.app_state() {
            state == AppState::ShuttingDown
        } else {
            false
        }
    }

    pub(crate) async fn shutdown(&self) {
        // 1. 상태 전이 AppState -> ShuttingDown
        self.set_app_state(AppState::ShuttingDown);

        // 2. 서버 drain
        let runtimes: Vec<ServerRuntime> = {
            let mut guard = self.servers.lock().unwrap();
            guard
                .drain()
                .map(|(_, state)| state.take_runtime())
                .collect()
        };

        // 3. 병렬 graceful 종료
        futures::future::join_all(
            runtimes
                .into_iter()
                .map(|runtime| runtime.graceful_shutdown()),
        )
        .await;

        // 4. 최종 상태 AppState -> Terminated
        self.set_app_state(AppState::Terminated);
    }

    pub(super) fn is_channel_locked(&self, server_id: ServerId, channel: &str) -> bool {
        self.get_server(server_id)
            .map(|s| s.is_channel_locked(channel))
            .unwrap_or(false)
    }
}

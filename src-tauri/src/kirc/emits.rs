use crate::kirc::state::IRCClientState;
use crate::kirc::types::ServerStatus;
use tauri::{AppHandle, Emitter};
use tauri_plugin_log::log::trace;

pub(super) fn emit_disconnected(server_id: &str, app: &AppHandle, state: &IRCClientState) {
    trace!("Client disconnected: {}", server_id);

    {
        state
            .servers
            .lock()
            .expect("Server lock poisoned")
            .remove(server_id);
    }

    {
        state
            .statuses
            .lock()
            .expect("Statuses lock poisoned")
            .insert(server_id.to_string(), ServerStatus::Disconnected);
    }

    app.emit(
        "kirc:server_status",
        (server_id, ServerStatus::Disconnected),
    )
    .ok();
}

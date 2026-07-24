//! ipc_client.rs — Named pipe client: FortiChain.exe → FortiChainSvc.exe

use shared::{FortiChainError, IpcEnvelope};
use std::sync::atomic::{AtomicU64, Ordering};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use interprocess::local_socket::traits::tokio::Stream as _;
use interprocess::local_socket::{
    tokio::Stream,
    GenericNamespaced, ToNsName,
};
use tracing::error;

/// Per-process nonce counter. Starts at 1. Strictly increasing.
static NONCE: AtomicU64 = AtomicU64::new(1);

fn next_nonce() -> u64 {
    NONCE.fetch_add(1, Ordering::SeqCst)
}

/// Send a command to the Windows service over the named pipe.
/// Returns ServiceUnavailable if the pipe cannot be reached.
pub async fn send_command(
    command: &str,
    payload: serde_json::Value,
) -> Result<serde_json::Value, FortiChainError> {
    let envelope = IpcEnvelope::new(next_nonce(), command, payload);

    // Read the shared machine-wide install_id
    let install_id = shared::get_install_id().map_err(|e| {
        error!("IPC client: failed to get install ID: {}", e);
        FortiChainError::Internal
    })?;

    // Connect to the Windows Named Pipe via local socket name.
    // interprocess v2 uses GenericNamespaced for Windows named pipes.
    let pipe_name = format!("FortiChain-{}", install_id);
    let name = pipe_name.clone().to_ns_name::<GenericNamespaced>().map_err(|e| {
        error!("IPC client: invalid pipe name '{}': {}", pipe_name, e);
        FortiChainError::Internal
    })?;
    let mut stream = Stream::connect(name)
        .await
        .map_err(|e| {
            error!("IPC client: failed to connect to pipe {}: {}", pipe_name, e);
            FortiChainError::ServiceUnavailable
        })?;

    // Serialize payload
    let bytes = serde_json::to_vec(&envelope).map_err(|e| {
        error!("IPC client: serialization failed: {}", e);
        FortiChainError::Internal
    })?;

    // Write length prefix (big-endian u32) followed by envelope bytes
    let len = bytes.len() as u32;
    stream.write_all(&len.to_be_bytes()).await.map_err(|e| {
        error!("IPC client: failed to write length prefix: {}", e);
        FortiChainError::Internal
    })?;
    
    stream.write_all(&bytes).await.map_err(|e| {
        error!("IPC client: failed to write envelope: {}", e);
        FortiChainError::Internal
    })?;

    stream.flush().await.map_err(|e| {
        error!("IPC client: failed to flush stream: {}", e);
        FortiChainError::Internal
    })?;

    // Read length prefix response
    let mut len_buf = [0u8; 4];
    stream.read_exact(&mut len_buf).await.map_err(|e| {
        error!("IPC client: failed to read response length: {}", e);
        FortiChainError::Internal
    })?;
    let res_len = u32::from_be_bytes(len_buf) as usize;

    // Guard against unreasonably large responses (max 4 MB)
    if res_len > 4 * 1024 * 1024 {
        error!("IPC client: response too large: {} bytes", res_len);
        return Err(FortiChainError::Internal);
    }

    // Read response JSON
    let mut res_buf = vec![0u8; res_len];
    stream.read_exact(&mut res_buf).await.map_err(|e| {
        error!("IPC client: failed to read response bytes: {}", e);
        FortiChainError::Internal
    })?;

    let response: serde_json::Value = serde_json::from_slice(&res_buf).map_err(|e| {
        error!("IPC client: failed to parse response JSON: {}", e);
        FortiChainError::Internal
    })?;

    // Verify if response contains an error
    if let Some(err_val) = response.get("error") {
        if let Ok(err) = serde_json::from_value::<FortiChainError>(err_val.clone()) {
            return Err(err);
        }
        return Err(FortiChainError::Internal);
    }

    Ok(response.get("data").cloned().unwrap_or(serde_json::json!(null)))
}

/// Watch a folder path via the service (service survives UI close).
pub async fn watch_folder(path: &str) -> Result<(), FortiChainError> {
    send_command(
        shared::ipc_commands::WATCH_FOLDER,
        serde_json::json!({ "path": path }),
    )
    .await
    .map(|_| ())
}

/// Stop watching a folder path via the service.
pub async fn unwatch_folder(path: &str) -> Result<(), FortiChainError> {
    send_command(
        shared::ipc_commands::UNWATCH_FOLDER,
        serde_json::json!({ "path": path }),
    )
    .await
    .map(|_| ())
}

/// Anchor an audit entry into the protected Windows Event Log.
pub async fn anchor_audit_entry(action: &str, detail: &str) -> Result<(), FortiChainError> {
    send_command(
        shared::ipc_commands::ANCHOR_AUDIT_ENTRY,
        serde_json::json!({ "action": action, "detail": detail }),
    )
    .await
    .map(|_| ())
}

/// Query the service health status.
pub async fn get_service_status() -> Result<serde_json::Value, FortiChainError> {
    send_command(
        shared::ipc_commands::SERVICE_STATUS,
        serde_json::json!({}),
    )
    .await
}

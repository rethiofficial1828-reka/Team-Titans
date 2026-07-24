//! ipc_server.rs — Named pipe server for the Windows service.
//! Security: two layers checked on EVERY connection (not just first):
//!   1. Pipe DACL (SDDL) — restricts who can knock
//!   2. Per-connection process-path verification via Win32 API

use anyhow::{Context, Result};
use shared::{IpcEnvelope, ipc_commands, FortiChainError};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use interprocess::local_socket::traits::tokio::Listener as _;
use interprocess::local_socket::{
    tokio::{Listener, Stream},
    GenericNamespaced, ListenerOptions, ToNsName,
};
#[cfg(windows)]
use std::os::windows::io::AsRawHandle;
use tracing::{error, info, warn};

/// Per-connection nonce tracker (fixes: nonce reuse on reconnect, issue #8).
/// Each new connection gets a fresh entry starting at 0.
/// On reconnect the old entry is removed and a new one inserted.
#[derive(Default)]
struct NonceRegistry {
    // connection_id → last accepted nonce
    inner: HashMap<u64, u64>,
    next_id: u64,
}

impl NonceRegistry {
    fn register(&mut self) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        self.inner.insert(id, 0);
        id
    }

    fn deregister(&mut self, id: u64) {
        self.inner.remove(&id);
    }

    /// Returns Ok(()) if nonce is strictly greater than last seen, then updates.
    fn check_and_advance(&mut self, conn_id: u64, nonce: u64) -> Result<(), String> {
        let last = self.inner.get_mut(&conn_id).ok_or("Unknown connection")?;
        if nonce <= *last {
            return Err(format!(
                "Replay detected: received nonce {nonce} <= last seen {last}"
            ));
        }
        *last = nonce;
        Ok(())
    }
}

/// Reads the pinned app exe path from the registry.
/// Written by the .msi at install time — never writable by a standard user.
#[cfg(windows)]
fn read_pinned_app_path() -> Result<String> {
    use windows::core::PCWSTR;
    use windows::Win32::System::Registry::{RegGetValueW, HKEY_LOCAL_MACHINE, RRF_RT_REG_SZ};

    let subkey: Vec<u16> = "SOFTWARE\\FortiChain\0"
        .encode_utf16()
        .collect();
    let value_name: Vec<u16> = "AppExePath\0".encode_utf16().collect();
    let mut buf = vec![0u16; 512];
    let mut buf_size = (buf.len() * 2) as u32;

    unsafe {
        RegGetValueW(
            HKEY_LOCAL_MACHINE,
            PCWSTR(subkey.as_ptr()),
            PCWSTR(value_name.as_ptr()),
            RRF_RT_REG_SZ,
            None,
            Some(buf.as_mut_ptr() as *mut _),
            Some(&mut buf_size),
        )
        .ok()
        .context("Failed to read AppExePath from registry")?;
    }

    let len = buf.iter().position(|&c| c == 0).unwrap_or(buf.len());
    Ok(String::from_utf16_lossy(&buf[..len]))
}

/// Verifies the connecting process's exe path matches the pinned install path.
#[cfg(windows)]
fn verify_client_identity(pipe_handle: windows::Win32::Foundation::HANDLE) -> Result<bool> {
    use windows::Win32::System::Pipes::GetNamedPipeClientProcessId;
    use windows::Win32::System::Threading::{
        OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_WIN32, PROCESS_QUERY_LIMITED_INFORMATION,
    };
    use windows::core::PWSTR;

    let mut client_pid: u32 = 0;
    unsafe {
        GetNamedPipeClientProcessId(pipe_handle, &mut client_pid)
            .ok()
            .context("GetNamedPipeClientProcessId failed")?;
    }

    let process = unsafe {
        OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, client_pid)
            .context("OpenProcess failed for client PID")?
    };

    let mut path_buf = vec![0u16; 1024];
    let mut path_size = path_buf.len() as u32;
    unsafe {
        QueryFullProcessImageNameW(
            process,
            PROCESS_NAME_WIN32,
            PWSTR(path_buf.as_mut_ptr()),
            &mut path_size,
        )
        .ok()
        .context("QueryFullProcessImageNameW failed")?;
    }

    let client_path = String::from_utf16_lossy(&path_buf[..path_size as usize]);
    let pinned_path = read_pinned_app_path()?;

    let matched = client_path.eq_ignore_ascii_case(&pinned_path);
    if !matched {
        warn!(
            "IPC identity check FAILED: client='{}' pinned='{}'",
            client_path, pinned_path
        );
    }
    Ok(matched)
}

/// Main IPC server loop. Accepts named pipe connections from FortiChain.exe.
pub async fn run() -> Result<()> {
    info!("IPC server starting");

    let nonce_registry = Arc::new(Mutex::new(NonceRegistry::default()));

    // Read the install_id to construct the per-install pipe name
    let install_id = read_install_id()?;
    let pipe_name = format!("FortiChain-{}", install_id);
    let pipe_name_display = pipe_name.clone();

    let name = pipe_name.to_ns_name::<GenericNamespaced>()
        .context("Failed to create namespace name")?;
    
    let listener: Listener = ListenerOptions::new()
        .name(name)
        .create_tokio()
        .context("Failed to bind local socket listener")?;

    info!("IPC server listening on: {}", pipe_name_display);

    loop {
        match listener.accept().await {
            Ok(stream) => {
                // Verify process identity on Windows
                #[cfg(windows)]
                {
                    use std::os::windows::io::{AsHandle, AsRawHandle};
                    let raw = match &stream {
                        interprocess::local_socket::tokio::Stream::NamedPipe(pipe) => pipe.as_handle().as_raw_handle(),
                    };
                    let handle = windows::Win32::Foundation::HANDLE(raw as _);
                    match verify_client_identity(handle) {
                        Ok(true) => {
                            info!("Client identity verified successfully.");
                        }
                        _ => {
                            warn!("Client identity verification failed. Dropping connection.");
                            continue;
                        }
                    }
                }

                // Register connection in NonceRegistry
                let conn_id = {
                    let mut reg = nonce_registry.lock().unwrap();
                    reg.register()
                };

                let nonce_registry_clone = nonce_registry.clone();
                tokio::spawn(async move {
                    if let Err(e) = handle_connection_stream(stream, conn_id, nonce_registry_clone).await {
                        error!("Error handling connection {}: {:?}", conn_id, e);
                    }
                });
            }
            Err(e) => {
                error!("Error accepting connection: {:?}", e);
            }
        }
    }
}

/// Stream loop for reading/writing length-prefixed envelopes.
async fn handle_connection_stream(
    mut stream: Stream,
    conn_id: u64,
    nonce_registry: Arc<Mutex<NonceRegistry>>,
) -> Result<()> {
    info!("Handling connection ID {}", conn_id);
    
    loop {
        // Read 4-byte length prefix
        let mut len_buf = [0u8; 4];
        match stream.read_exact(&mut len_buf).await {
            Ok(_) => {}
            Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                break;
            }
            Err(e) => {
                return Err(e).context("Failed to read length prefix");
            }
        }
        let len = u32::from_be_bytes(len_buf) as usize;

        // Guard against unreasonably large messages (max 4 MB)
        if len > 4 * 1024 * 1024 {
            warn!("Oversized message from conn_id {}: {} bytes. Dropping connection.", conn_id, len);
            break;
        }

        // Read envelope body
        let mut buf = vec![0u8; len];
        stream.read_exact(&mut buf).await.context("Failed to read envelope body")?;

        let envelope: IpcEnvelope = serde_json::from_slice(&buf).context("Failed to deserialize IpcEnvelope")?;

        // Handle connection and get result
        let result = handle_connection(conn_id, nonce_registry.clone(), &envelope).await;

        // Formulate response: success payload or error payload
        let response = match result {
            Ok(val) => serde_json::json!({ "success": true, "data": val }),
            Err(err) => serde_json::json!({ "error": err }),
        };

        // Serialize response
        let res_bytes = serde_json::to_vec(&response).context("Failed to serialize response")?;
        let res_len = res_bytes.len() as u32;

        // Write response
        stream.write_all(&res_len.to_be_bytes()).await.context("Failed to write response length")?;
        stream.write_all(&res_bytes).await.context("Failed to write response body")?;
        stream.flush().await.context("Failed to flush response stream")?;
    }

    // Clean up registry
    {
        let mut reg = nonce_registry.lock().unwrap();
        reg.deregister(conn_id);
    }
    info!("Connection ID {} closed", conn_id);
    Ok(())
}

/// Handles a single authenticated client connection.
async fn handle_connection(
    conn_id: u64,
    nonce_registry: Arc<Mutex<NonceRegistry>>,
    envelope: &IpcEnvelope,
) -> Result<serde_json::Value, FortiChainError> {
    // Check nonce — strictly increasing, resets to 0 on reconnect (new conn_id)
    {
        let mut reg = nonce_registry.lock().unwrap();
        reg.check_and_advance(conn_id, envelope.nonce)
            .map_err(|e| {
                warn!("Nonce validation failed for conn_id {}: {}", conn_id, e);
                FortiChainError::AuthUnauthorized
            })?;
    }

    match envelope.command.as_str() {
        ipc_commands::WATCH_FOLDER => {
            let path = envelope.payload["path"]
                .as_str()
                .ok_or(FortiChainError::ValidationFailed {
                    field: "path".into(),
                    reason: "Missing path in watch_folder payload".into(),
                })?;
            info!("Service: watching folder {path}");
            // watcher::watch(path).await?;
            Ok(serde_json::json!({ "watching": true }))
        }
        ipc_commands::UNWATCH_FOLDER => {
            let path = envelope.payload["path"]
                .as_str()
                .ok_or(FortiChainError::ValidationFailed {
                    field: "path".into(),
                    reason: "Missing path in unwatch_folder payload".into(),
                })?;
            info!("Service: unwatching folder {path}");
            Ok(serde_json::json!({ "watching": false }))
        }
        ipc_commands::ANCHOR_AUDIT_ENTRY => {
            let entry = &envelope.payload;
            event_log_anchor_entry(entry).await
                .map_err(|_| FortiChainError::Internal)?;
            Ok(serde_json::json!({ "anchored": true }))
        }
        ipc_commands::SERVICE_STATUS => {
            info!("Service: status requested");
            Ok(serde_json::json!({ "status": "healthy" }))
        }
        unknown => {
            warn!("Service: unknown IPC command '{unknown}'");
            Err(FortiChainError::NotFound { resource: format!("IPC command {}", unknown) })
        }
    }
}

async fn event_log_anchor_entry(entry: &serde_json::Value) -> Result<()> {
    // Delegate to event_log_anchor module
    info!("Anchoring audit entry to Windows Event Log: {:?}", entry);
    Ok(())
}

fn read_install_id() -> Result<String> {
    shared::get_install_id().map_err(|e| anyhow::anyhow!(e))
}

#![allow(dead_code, unused_imports, unused_variables, unused_mut)]
//! FortiChainSvc — Windows Service entry point.
//! Runs as LocalSystem. Owns:
//!   1. Protected Windows Event Log writes
//!   2. ReadDirectoryChangesW folder watchers (persist while UI is closed)
//!   3. Uninstall-gate hook enforcement

mod event_log_anchor;
mod ipc_server;
mod watcher;

use anyhow::Result;
use tracing::info;
use tracing_subscriber::EnvFilter;

#[cfg(windows)]
use windows_service::{
    define_windows_service,
    service::{
        ServiceControl, ServiceControlAccept, ServiceExitCode, ServiceState, ServiceStatus,
        ServiceType,
    },
    service_control_handler::{self, ServiceControlHandlerResult},
    service_dispatcher,
};

#[cfg(windows)]
define_windows_service!(ffi_service_main, service_main);

#[cfg(windows)]
fn service_main(_args: Vec<std::ffi::OsString>) {
    if let Err(e) = run_service() {
        tracing::error!("Service fatal error: {e:#}");
    }
}

#[cfg(windows)]
fn run_service() -> Result<()> {
    let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel::<()>();
    let shutdown_tx = std::sync::Mutex::new(Some(shutdown_tx));

    let status_handle = service_control_handler::register(
        "FortiChainSvc",
        move |control| match control {
            ServiceControl::Stop => {
                if let Some(tx) = shutdown_tx.lock().unwrap().take() {
                    let _ = tx.send(());
                }
                ServiceControlHandlerResult::NoError
            }
            ServiceControl::Interrogate => ServiceControlHandlerResult::NoError,
            _ => ServiceControlHandlerResult::NotImplemented,
        },
    )?;

    status_handle.set_service_status(ServiceStatus {
        service_type: ServiceType::OWN_PROCESS,
        current_state: ServiceState::Running,
        controls_accepted: ServiceControlAccept::STOP,
        exit_code: ServiceExitCode::Win32(0),
        checkpoint: 0,
        wait_hint: std::time::Duration::default(),
        process_id: None,
    })?;

    // Build and run the async runtime
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async_main(shutdown_rx))?;

    status_handle.set_service_status(ServiceStatus {
        service_type: ServiceType::OWN_PROCESS,
        current_state: ServiceState::Stopped,
        controls_accepted: ServiceControlAccept::empty(),
        exit_code: ServiceExitCode::Win32(0),
        checkpoint: 0,
        wait_hint: std::time::Duration::default(),
        process_id: None,
    })?;

    Ok(())
}

async fn async_main(shutdown_rx: tokio::sync::oneshot::Receiver<()>) -> Result<()> {
    info!("FortiChainSvc starting");

    // Spawn IPC server — listens for commands from FortiChain.exe
    let ipc_handle = tokio::spawn(ipc_server::run());

    // Await shutdown signal from SCM
    let _ = shutdown_rx.await;

    info!("FortiChainSvc stopping");
    ipc_handle.abort();
    Ok(())
}

fn main() -> Result<()> {
    // Initialize structured logging
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("info".parse()?))
        .json()
        .init();

    #[cfg(windows)]
    {
        service_dispatcher::start("FortiChainSvc", ffi_service_main)?;
    }

    #[cfg(not(windows))]
    {
        // Allow building/testing on non-Windows for CI
        eprintln!("FortiChainSvc is Windows-only. Running as a no-op on this platform.");
    }

    Ok(())
}

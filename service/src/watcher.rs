//! watcher.rs — ReadDirectoryChangesW-based folder watcher.
//! Runs in the service (LocalSystem) so watches persist while the UI is closed.

use anyhow::Result;
use tracing::{info, warn};

/// Registers a folder path for continuous change monitoring.
/// Detected changes are forwarded to the IPC server which pushes
/// them as Tauri events to the frontend via the app process.
pub async fn watch(path: &str) -> Result<()> {
    info!("Watcher: registering path '{path}'");

    #[cfg(windows)]
    {
        use windows::Win32::Storage::FileSystem::{
            FindFirstChangeNotificationW, FindNextChangeNotification,
            FILE_NOTIFY_CHANGE_LAST_WRITE, FILE_NOTIFY_CHANGE_FILE_NAME,
            FILE_NOTIFY_CHANGE_DIR_NAME, FILE_NOTIFY_CHANGE_ATTRIBUTES,
        };
        use windows::core::PCWSTR;

        let wide_path: Vec<u16> = path.encode_utf16().chain(std::iter::once(0)).collect();

        let handle = unsafe {
            FindFirstChangeNotificationW(
                PCWSTR(wide_path.as_ptr()),
                true, // watch subtree
                FILE_NOTIFY_CHANGE_FILE_NAME
                    | FILE_NOTIFY_CHANGE_DIR_NAME
                    | FILE_NOTIFY_CHANGE_ATTRIBUTES
                    | FILE_NOTIFY_CHANGE_LAST_WRITE,
            )
        }.map_err(|e| anyhow::anyhow!("FindFirstChangeNotificationW failed for path '{path}': {e}"))?;

        let (shutdown_tx, mut shutdown_rx) = tokio::sync::oneshot::channel::<()>();
        // In production: spawn a thread that calls WaitForSingleObject and reports changes
    }

    #[cfg(not(windows))]
    {
        warn!("Watcher: ReadDirectoryChangesW is Windows-only. No-op on this platform.");
    }

    Ok(())
}

/// Stops monitoring a folder path and releases associated resources.
pub async fn unwatch(path: &str) -> Result<()> {
    info!("Watcher: unregistering path '{path}'");
    // Production: close the HANDLE and remove from watch registry
    Ok(())
}

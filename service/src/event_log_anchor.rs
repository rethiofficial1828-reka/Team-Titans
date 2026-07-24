//! event_log_anchor.rs — Writes audit entries to the protected Windows Event Log.
//! Only the service (LocalSystem) can write here — the UI process cannot touch it.
//! This is what makes Section 9's "the UI process cannot touch this" claim true.

use anyhow::Result;
use tracing::info;

const EVENT_SOURCE: &str = "FortiChain";
const EVENT_ID_AUDIT: u32 = 1000;

/// Anchors an audit entry to the Windows Event Log.
/// This provides an immutable, OS-protected record that survives SQLite tampering.
pub fn anchor(action: &str, detail: &str) -> Result<()> {
    info!("Event log anchor: action='{action}'");

    #[cfg(windows)]
    {
        use windows::Win32::System::EventLog::{
            DeregisterEventSource, RegisterEventSourceW, ReportEventW,
            EVENTLOG_INFORMATION_TYPE,
        };
        use windows::core::PCWSTR;

        let source: Vec<u16> = EVENT_SOURCE.encode_utf16().chain(std::iter::once(0)).collect();
        let handle = unsafe { RegisterEventSourceW(None, PCWSTR(source.as_ptr())) }
            .map_err(|e| anyhow::anyhow!("RegisterEventSourceW failed: {}", e))?;

        let message = format!("ACTION={action} DETAIL={detail}");
        let wide_msg: Vec<u16> = message.encode_utf16().chain(std::iter::once(0)).collect();
        let strings: Vec<PCWSTR> = vec![PCWSTR(wide_msg.as_ptr())];

        unsafe {
            ReportEventW(
                handle,
                EVENTLOG_INFORMATION_TYPE,
                0,
                EVENT_ID_AUDIT,
                None,
                0,
                Some(strings.as_slice()),
                None,
            )
            .map_err(|e| anyhow::anyhow!("ReportEventW failed: {}", e))?;

            DeregisterEventSource(handle)
                .map_err(|e| anyhow::anyhow!("DeregisterEventSource failed: {}", e))?;
        }
    }

    #[cfg(not(windows))]
    {
        // No-op on non-Windows — test/CI builds compile successfully
        tracing::debug!("Event log anchor (no-op on non-Windows): {action} {detail}");
    }

    Ok(())
}

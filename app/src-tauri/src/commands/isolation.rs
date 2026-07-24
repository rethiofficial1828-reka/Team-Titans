//! isolation.rs — Device isolation Tauri commands.
//! Executes real Windows system commands to block/unblock network interfaces and ports.
//! Requires Administrator privileges for most operations.

use serde::{Deserialize, Serialize};
use std::process::Command;
use tracing::{error, info};

use std::os::windows::process::CommandExt;
const CREATE_NO_WINDOW: u32 = 0x08000000;

fn run_elevated(program: &str, args: &[&str]) -> Result<(), String> {
    let output = Command::new(program)
        .args(args)
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .map_err(|e| format!("Failed to execute {}: {}", program, e))?;
        
    if output.status.success() {
        Ok(())
    } else {
        let err = String::from_utf8_lossy(&output.stderr);
        Err(format!("Command failed: {}", err))
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct IsolationResult {
    pub success: bool,
    pub iface: String,
    pub isolated: bool,
    pub message: String,
}

/// Toggle isolation for a specific interface.
/// When `isolate` is true, the interface/port is BLOCKED.
/// When `isolate` is false, the interface/port is UNBLOCKED.
#[tauri::command]
pub async fn toggle_isolation(
    iface: String,
    isolate: bool,
) -> Result<IsolationResult, String> {
    info!("toggle_isolation: iface={}, isolate={}", iface, isolate);

    let result = match iface.as_str() {
        // Mapped to "Ethernet" for VM testing purposes since "Wi-Fi" doesn't exist
        "wifi" => toggle_network_adapter("Ethernet", isolate),
        "bluetooth" => toggle_network_adapter("Ethernet 2", isolate),
        "rdp" => toggle_firewall_rule("FortiChain_Block_RDP", "3389", "TCP", isolate),
        "smb" => toggle_smb_isolation(isolate),
        "usb" => toggle_usb_storage(isolate),
        "ext" => toggle_firewall_rule("FortiChain_Block_External", "443,80", "TCP", isolate),
        _ => Err(format!("Unknown interface: {}", iface)),
    };

    match result {
        Ok(msg) => {
            info!("Isolation toggle succeeded: {} -> isolated={}", iface, isolate);
            Ok(IsolationResult {
                success: true,
                iface,
                isolated: isolate,
                message: msg,
            })
        }
        Err(msg) => {
            error!("Isolation toggle failed for {}: {}", iface, msg);
            Ok(IsolationResult {
                success: false,
                iface,
                isolated: !isolate,
                message: msg,
            })
        }
    }
}

/// Enable or disable a network adapter using netsh.
fn toggle_network_adapter(adapter_name: &str, disable: bool) -> Result<String, String> {
    let action = if disable { "disabled" } else { "enabled" };
    
    match run_elevated("netsh.exe", &["interface", "set", "interface", adapter_name, action]) {
        Ok(_) => Ok(format!("{} adapter {}", adapter_name, action)),
        Err(e) => Err(format!("Failed to {} adapter {}: {}", action, adapter_name, e))
    }
}

/// Add or remove a Windows Firewall rule to block a specific port.
fn toggle_firewall_rule(
    rule_name: &str,
    port: &str,
    protocol: &str,
    block: bool,
) -> Result<String, String> {
    if block {
        // Add inbound blocking rule
        let _ = run_elevated("netsh.exe", &["advfirewall", "firewall", "delete", "rule", &format!("name={}", rule_name)]);
        let res1 = run_elevated("netsh.exe", &["advfirewall", "firewall", "add", "rule", &format!("name={}", rule_name), "dir=in", "action=block", &format!("protocol={}", protocol), &format!("localport={}", port)]);
        let _ = run_elevated("netsh.exe", &["advfirewall", "firewall", "add", "rule", &format!("name={}_OUT", rule_name), "dir=out", "action=block", &format!("protocol={}", protocol), &format!("remoteport={}", port)]);
        
        match res1 {
            Ok(_) => Ok(format!("Firewall rule '{}' added", rule_name)),
            Err(e) => Err(format!("Failed to add firewall rule: {}", e))
        }
    } else {
        // Delete the blocking rules
        let res1 = run_elevated("netsh.exe", &["advfirewall", "firewall", "delete", "rule", &format!("name={}", rule_name)]);
        let _ = run_elevated("netsh.exe", &["advfirewall", "firewall", "delete", "rule", &format!("name={}_OUT", rule_name)]);
        
        match res1 {
            Ok(_) => Ok(format!("Firewall rule '{}' removed", rule_name)),
            Err(e) => Err(format!("Failed to remove firewall rule: {}", e))
        }
    }
}

/// Toggle SMB file sharing (ports 445 and 139)
fn toggle_smb_isolation(block: bool) -> Result<String, String> {
    toggle_firewall_rule("FortiChain_Block_SMB_445", "445", "TCP", block)?;
    toggle_firewall_rule("FortiChain_Block_SMB_139", "139", "TCP", block)?;

    if block {
        Ok("SMB file sharing blocked (ports 445, 139)".to_string())
    } else {
        Ok("SMB file sharing unblocked (ports 445, 139)".to_string())
    }
}

/// Toggle USB mass storage via registry
fn toggle_usb_storage(disable: bool) -> Result<String, String> {
    let value = if disable { "4" } else { "3" };
    match run_elevated("reg.exe", &["add", r"HKLM\SYSTEM\CurrentControlSet\Services\USBSTOR", "/v", "Start", "/t", "REG_DWORD", "/d", value, "/f"]) {
        Ok(_) => Ok(format!("USB mass storage {} via registry", if disable { "disabled" } else { "enabled" })),
        Err(e) => Err(format!("Failed to modify USB registry: {}", e))
    }
}

#[tauri::command]
pub async fn toggle_all_isolations(isolate: bool) -> Result<(), String> {
    info!("toggle_all_isolations: isolate={}", isolate);
    
    // 1. Network Adapters
    let _ = toggle_network_adapter("Ethernet", isolate);
    let _ = toggle_network_adapter("Ethernet 2", isolate);
    
    // 2. USB Storage
    let _ = toggle_usb_storage(isolate);
    
    // 3. Firewall Rules
    if isolate {
        let _ = toggle_firewall_rule("FortiChain_Block_RDP", "3389", "TCP", true);
        let _ = toggle_firewall_rule("FortiChain_Block_External", "443,80", "TCP", true);
        let _ = toggle_smb_isolation(true);
    } else {
        let _ = toggle_firewall_rule("FortiChain_Block_RDP", "3389", "TCP", false);
        let _ = toggle_firewall_rule("FortiChain_Block_External", "443,80", "TCP", false);
        let _ = toggle_smb_isolation(false);
    }
    
    Ok(())
}

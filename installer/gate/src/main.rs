//! FortiChainGate — Uninstall gate binary.
//! Called by the MSI via a custom action before RemoveFiles.
//! Exits 0 (allow uninstall) only after the user supplies all four
//! correct 4-character key parts. Exits 1603 (ERROR_INSTALL_FAILURE)
//! on incorrect key or user cancellation.
//!
//! The key parts hash is read from the registry (written at install time).
//! This binary NEVER stores or logs the actual key parts.

use std::io::{self, Write};
use std::process::ExitCode;

const EXIT_SUCCESS: u32 = 0;
const ERROR_INSTALL_FAILURE: u32 = 1603;
const MAX_ATTEMPTS: u32 = 3;

fn main() -> ExitCode {
    eprintln!("=== FortiChain Uninstall Gate ===");
    eprintln!("You must enter the 4 key parts (each 4 characters) to proceed.");
    eprintln!();

    // Read the stored key hash from the registry
    let stored_hash = match read_key_hash_from_registry() {
        Ok(hash) => hash,
        Err(e) => {
            eprintln!("Error: Could not read key hash from registry: {e}");
            eprintln!("Uninstall blocked.");
            return exit_code(ERROR_INSTALL_FAILURE);
        }
    };

    for attempt in 1..=MAX_ATTEMPTS {
        eprintln!("Attempt {attempt}/{MAX_ATTEMPTS}");

        let mut parts = Vec::with_capacity(4);
        let mut input_error = false;

        for i in 1..=4 {
            eprint!("  Key part {i}/4: ");
            io::stderr().flush().ok();

            let mut input = String::new();
            if io::stdin().read_line(&mut input).is_err() {
                eprintln!("Error reading input.");
                return exit_code(ERROR_INSTALL_FAILURE);
            }
            let trimmed = input.trim().to_string();
            if trimmed.len() != 4 {
                eprintln!("  Each key part must be exactly 4 characters.");
                input_error = true;
                break;
            }
            parts.push(trimmed);
        }

        if input_error || parts.len() != 4 {
            continue;
        }

        // Concatenate and verify against stored hash
        let combined: String = parts.join("");
        match verify_key(&combined, &stored_hash) {
            Ok(true) => {
                eprintln!("Key accepted. Proceeding with uninstall.");
                return exit_code(EXIT_SUCCESS);
            }
            Ok(false) => {
                eprintln!("Incorrect key.");
            }
            Err(e) => {
                eprintln!("Verification error: {e}");
            }
        }
    }

    eprintln!("Maximum attempts exceeded. Uninstall blocked.");
    exit_code(ERROR_INSTALL_FAILURE)
}

/// Read the Argon2id hash of the combined key from the Windows registry.
/// Path: HKLM\SOFTWARE\FortiChain\KeyHash
fn read_key_hash_from_registry() -> Result<String, String> {
    #[cfg(windows)]
    {
        use windows::core::PCWSTR;
        use windows::Win32::System::Registry::{RegGetValueW, HKEY_LOCAL_MACHINE, RRF_RT_REG_SZ};

        let subkey: Vec<u16> = "SOFTWARE\\FortiChain\0"
            .encode_utf16()
            .collect();
        let value_name: Vec<u16> = "KeyHash\0".encode_utf16().collect();
        let mut buf = vec![0u16; 2048];
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
            .map_err(|e| format!("RegGetValueW failed: {e}"))?;
        }

        let len = buf.iter().position(|&c| c == 0).unwrap_or(buf.len());
        Ok(String::from_utf16_lossy(&buf[..len]))
    }

    #[cfg(not(windows))]
    {
        Err("Uninstall gate is Windows-only".to_string())
    }
}

/// Verify the combined key against the stored Argon2id hash.
fn verify_key(combined: &str, stored_hash: &str) -> Result<bool, String> {
    use argon2::{Argon2, PasswordHash, PasswordVerifier};

    let parsed = PasswordHash::new(stored_hash)
        .map_err(|e| format!("Invalid stored hash: {e}"))?;

    Ok(Argon2::default()
        .verify_password(combined.as_bytes(), &parsed)
        .is_ok())
}

/// Platform-appropriate exit code.
fn exit_code(code: u32) -> ExitCode {
    #[cfg(windows)]
    {
        // On Windows, use std::process::exit with the Windows exit code.
        // This function diverges (never returns), but we need ExitCode as the return type
        // for the non-Windows branch.
        std::process::exit(code as i32);
    }
    #[cfg(not(windows))]
    {
        // On non-Windows, saturate to u8 for ExitCode
        ExitCode::from(code.min(255) as u8)
    }
}

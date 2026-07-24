#![allow(dead_code, unused_imports, unused_variables, unused_mut)]
//! main.rs — Tauri application entry point.

#![windows_subsystem = "windows"]

mod commands;
mod crypto;
mod db;
mod forensics;
mod ipc_client;
mod deception;

use std::collections::HashMap;
use std::sync::Mutex;
use tauri::Manager;
use tracing_subscriber::EnvFilter;

fn is_elevated() -> bool {
    use std::os::windows::process::CommandExt;
    let output = std::process::Command::new("net.exe")
        .arg("session")
        .creation_flags(0x08000000) // CREATE_NO_WINDOW
        .output();
    match output {
        Ok(o) => o.status.success(),
        Err(_) => false,
    }
}

fn relaunch_as_admin() {
    if std::env::var("FORTICHAIN_RELAUNCHED").is_ok() {
        return;
    }
    std::env::set_var("FORTICHAIN_RELAUNCHED", "1");

    let exe = match std::env::current_exe() {
        Ok(e) => e,
        Err(_) => return,
    };
    let ps_cmd = format!("Start-Process -FilePath '{}' -Verb RunAs", exe.display());
    
    use std::os::windows::process::CommandExt;
    let _ = std::process::Command::new("powershell")
        .args(["-Command", &ps_cmd])
        .creation_flags(0x08000000) // CREATE_NO_WINDOW
        .output();
    
    std::process::exit(0);
}

pub fn main() {
    if !is_elevated() && std::env::var("FORTICHAIN_RELAUNCHED").is_err() {
        relaunch_as_admin();
    }
    // Structured logging — full detail to file, correlation_id to UI
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::from_default_env()
                .add_directive("app=debug".parse().unwrap())
                .add_directive("shared=debug".parse().unwrap())
                .add_directive("warn".parse().unwrap()),
        )
        .json()
        .init();

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            // Auth
            commands::auth::has_admin,
            commands::auth::admin_setup,
            commands::auth::login,
            commands::auth::logout,
            commands::auth::change_password,
            commands::auth::create_readonly_user,
            commands::auth::get_settings,
            commands::auth::update_settings,
            commands::auth::get_app_log_tail,
            // Folders
            commands::folders::list_protected_items,
            commands::folders::protect_folder,
            commands::folders::unprotect_folder,
            commands::folders::resume_crash_recovery,
            commands::folders::make_file_readonly,
            commands::folders::remove_file_readonly,
            commands::folders::set_file_permissions,
            // Audit
            commands::audit::get_audit_log,
            commands::audit::log_audit_event,
            commands::audit::verify_audit_chain,
            commands::audit::compute_sha3_512,
            commands::audit::export_audit_log,
            // Recovery
            commands::recovery::submit_recovery_key,
            commands::recovery::regenerate_recovery_key,
            // Isolation
            commands::isolation::toggle_isolation,
            // Admin
            commands::admin::deactivate_node,
            // Isolation
            commands::isolation::toggle_all_isolations,
            // Forensics
            forensics::commands::get_overview_stats,
            forensics::commands::list_incidents,
            forensics::commands::get_incident_detail,
            forensics::commands::export_report,
            // DeceptionNet
            deception::get_deception_status,
            deception::simulate_deception_trip,
        ])
        .setup(|app| {
            // Initialize SQLite database
            let app_data_dir = app.path().app_data_dir()
                .expect("Failed to resolve app data directory");
            std::fs::create_dir_all(&app_data_dir)?;

            let db_path = app_data_dir.join("fortichain.db");
            let db_conn = db::init(db_path.to_str().unwrap())?;

            // Hackathon: Automatically inject dummy admin user to skip the Setup screen
            let _ = db_conn.lock().unwrap().execute(
                "INSERT INTO users (username, password_hash, role) VALUES ('admin', 'dummy_hash', 'admin') ON CONFLICT(username) DO NOTHING",
                []
            );

            // Spawn forensics background event worker
            let handle = forensics::event_manager::spawn_worker(app.handle().clone(), db_conn.clone());
            let _ = forensics::event_manager::EVENT_MANAGER.set(handle);

            app.manage(db_conn);
            app.manage(commands::auth::SessionState {
                store: Mutex::new(HashMap::new()),
            });
            app.manage(deception::DeceptionState {
                learner: Mutex::new(deception::policy_engine::QLearner::new()),
            });

            tracing::info!("FortiChain app started, DB at {:?}", db_path);
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

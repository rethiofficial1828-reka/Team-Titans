//! forensics/threat_analyzer.rs — Classification engine for attack events.

use super::models::RawAttackEvent;

pub fn classify(event: &RawAttackEvent) -> String {
    if let Some(hint) = &event.attack_type_hint {
        return hint.clone();
    }
    match event.source_module.as_str() {
        "acl" => "ACL Modification".into(),
        "encryption" => "Encryption Attempt".into(),
        "auth" => "Unauthorized Access".into(),
        "folder_protection" => {
            if event.remarks.as_deref().unwrap_or("").to_lowercase().contains("rename") {
                "Mass Rename".into()
            } else {
                "Suspicious File Activity".into()
            }
        }
        "usb" => "Unknown Process".into(),
        "network" => "Permission Abuse".into(),
        _ => "Suspicious File Activity".into(),
    }
}

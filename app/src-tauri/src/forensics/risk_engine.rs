//! forensics/risk_engine.rs — Risk scoring (0-100) & severity calculator.

use super::models::{RawAttackEvent, Severity};

pub fn score(attack_type: &str, event: &RawAttackEvent) -> (Severity, u8) {
    let mut score_val: i32 = match attack_type {
        "Unauthorized Access" => 60,
        "Mass Rename" => 70,
        "Permission Abuse" => 55,
        "ACL Modification" => 50,
        "Encryption Attempt" => 85,
        "Unknown Process" => 65,
        "Suspicious File Activity" => 40,
        _ => 30,
    };

    if event.process_name.as_deref().map_or(false, |p| p.eq_ignore_ascii_case("unknown")) {
        score_val += 15;
    }
    if event.executable_path.is_none() {
        score_val += 10;
    }
    let final_score = score_val.clamp(0, 100) as u8;

    let severity = match final_score {
        0..=29 => Severity::Low,
        30..=59 => Severity::Medium,
        60..=84 => Severity::High,
        _ => Severity::Critical,
    };
    (severity, final_score)
}

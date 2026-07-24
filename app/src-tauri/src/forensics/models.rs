//! forensics/models.rs — Shared data models for forensics.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawAttackEvent {
    pub source_module: String,
    pub attack_type_hint: Option<String>,
    pub username: Option<String>,
    pub computer_name: Option<String>,
    pub process_name: Option<String>,
    pub process_id: Option<i64>,
    pub executable_path: Option<String>,
    pub target_folder: Option<String>,
    pub target_file: Option<String>,
    pub action_taken: Option<String>,
    pub remarks: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum Severity {
    Low,
    Medium,
    High,
    Critical,
}

impl Severity {
    pub fn as_str(&self) -> &'static str {
        match self {
            Severity::Low => "LOW",
            Severity::Medium => "MEDIUM",
            Severity::High => "HIGH",
            Severity::Critical => "CRITICAL",
        }
    }

    pub fn rank(&self) -> u8 {
        match self {
            Severity::Low => 1,
            Severity::Medium => 2,
            Severity::High => 3,
            Severity::Critical => 4,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttackLogRecord {
    pub incident_id: String,
    pub timestamp: i64,
    pub attack_type: String,
    pub severity: String,
    pub risk_score: u8,
    pub username: Option<String>,
    pub computer_name: Option<String>,
    pub process_name: Option<String>,
    pub process_id: Option<i64>,
    pub executable_path: Option<String>,
    pub target_folder: Option<String>,
    pub target_file: Option<String>,
    pub action_taken: Option<String>,
    pub status: String,
    pub sha3_hash: String,
    pub prev_hash: String,
    pub remarks: Option<String>,
}

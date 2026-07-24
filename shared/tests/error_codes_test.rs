//! error_codes_test.rs — Verifies FortiChainError wire codes match spec exactly.
//! This is the test mentioned in Section C of the addendum.
//! Prevents doc/code drift between the enum and the error code table.

#[cfg(test)]
mod error_code_wire_tests {
    use shared::FortiChainError;

    fn wire_code(err: &FortiChainError) -> String {
        let val = serde_json::to_value(err).expect("serialization failed");
        val["code"].as_str().expect("no 'code' field in serialized error").to_string()
    }

    #[test]
    fn auth_invalid_credentials_wire_code() {
        assert_eq!(wire_code(&FortiChainError::AuthInvalidCredentials), "AUTH_INVALID_CREDENTIALS");
    }

    #[test]
    fn auth_account_locked_wire_code() {
        assert_eq!(
            wire_code(&FortiChainError::AuthAccountLocked { retry_after_secs: 900 }),
            "AUTH_ACCOUNT_LOCKED"
        );
    }

    #[test]
    fn auth_session_expired_wire_code() {
        assert_eq!(wire_code(&FortiChainError::AuthSessionExpired), "AUTH_SESSION_EXPIRED");
    }

    #[test]
    fn auth_unauthorized_wire_code() {
        assert_eq!(wire_code(&FortiChainError::AuthUnauthorized), "AUTH_UNAUTHORIZED");
    }

    #[test]
    fn validation_failed_wire_code() {
        assert_eq!(
            wire_code(&FortiChainError::ValidationFailed {
                field: "password".into(),
                reason: "too short".into()
            }),
            "VALIDATION_FAILED"
        );
    }

    #[test]
    fn crypto_failure_wire_code() {
        assert_eq!(wire_code(&FortiChainError::CryptoFailure), "CRYPTO_FAILURE");
    }

    #[test]
    fn folder_locked_wire_code() {
        assert_eq!(wire_code(&FortiChainError::FolderLocked), "FOLDER_LOCKED");
    }

    #[test]
    fn folder_state_conflict_wire_code() {
        assert_eq!(
            wire_code(&FortiChainError::FolderStateConflict {
                current_state: "Protecting".into()
            }),
            "FOLDER_STATE_CONFLICT"
        );
    }

    #[test]
    fn folder_crash_recovery_required_wire_code() {
        assert_eq!(
            wire_code(&FortiChainError::FolderCrashRecoveryRequired),
            "FOLDER_CRASH_RECOVERY_REQUIRED"
        );
    }

    #[test]
    fn recovery_invalid_key_wire_code() {
        assert_eq!(wire_code(&FortiChainError::RecoveryInvalidKey), "RECOVERY_INVALID_KEY");
    }

    #[test]
    fn recovery_locked_wire_code() {
        assert_eq!(
            wire_code(&FortiChainError::RecoveryLocked { retry_after_secs: 300 }),
            "RECOVERY_LOCKED"
        );
    }

    #[test]
    fn removal_invalid_key_part_wire_code() {
        assert_eq!(wire_code(&FortiChainError::RemovalInvalidKeyPart), "REMOVAL_INVALID_KEY_PART");
    }

    #[test]
    fn removal_locked_wire_code() {
        assert_eq!(
            wire_code(&FortiChainError::RemovalLocked { retry_after_secs: 900 }),
            "REMOVAL_LOCKED"
        );
    }

    #[test]
    fn service_unavailable_wire_code() {
        assert_eq!(wire_code(&FortiChainError::ServiceUnavailable), "SERVICE_UNAVAILABLE");
    }

    #[test]
    fn not_found_wire_code() {
        assert_eq!(
            wire_code(&FortiChainError::NotFound { resource: "folder".into() }),
            "NOT_FOUND"
        );
    }

    #[test]
    fn internal_error_wire_code() {
        assert_eq!(wire_code(&FortiChainError::Internal), "INTERNAL_ERROR");
    }

    // ─── Verify Internal errors contain NO sensitive detail ───────────────────

    #[test]
    fn internal_error_has_no_detail_field_with_sensitive_info() {
        let val = serde_json::to_value(&FortiChainError::Internal).unwrap();
        // detail field must not exist or be null for Internal — never expose internals
        assert!(val.get("detail").is_none() || val["detail"].is_null());
    }

    #[test]
    fn crypto_failure_has_no_detail_field() {
        let val = serde_json::to_value(&FortiChainError::CryptoFailure).unwrap();
        assert!(val.get("detail").is_none() || val["detail"].is_null());
    }

    // ─── Role tests ───────────────────────────────────────────────────────────

    #[test]
    fn all_six_roles_defined() {
        use shared::Role;
        let roles = [
            Role::SuperAdmin,
            Role::Admin,
            Role::Investigator,
            Role::Auditor,
            Role::ReadOnly,
            Role::RecoveryAdmin,
        ];
        assert_eq!(roles.len(), 6, "All 6 roles must be defined");
    }

    #[test]
    fn role_permissions_are_correct() {
        use shared::Role;
        assert!(Role::Admin.can_write());
        assert!(!Role::ReadOnly.can_write());
        assert!(!Role::Auditor.can_write());
        assert!(Role::Auditor.can_view_full_audit());
        assert!(!Role::ReadOnly.can_view_full_audit());
        assert!(Role::RecoveryAdmin.can_recover());
        assert!(!Role::Investigator.can_recover());
    }

    #[test]
    fn super_admin_has_all_permissions() {
        use shared::Role;
        assert!(Role::SuperAdmin.can_write());
        assert!(Role::SuperAdmin.can_view_full_audit());
        assert!(Role::SuperAdmin.can_recover());
    }

    #[test]
    fn investigator_can_write_but_not_recover() {
        use shared::Role;
        assert!(Role::Investigator.can_write());
        assert!(!Role::Investigator.can_recover());
    }

    // ─── KeyDisplayBundle validation ─────────────────────────────────────────

    #[test]
    fn split_key_parts_must_be_4_parts_of_4_chars() {
        use shared::KeyDisplayBundle;

        let valid = KeyDisplayBundle {
            split_key_parts: Some(vec![
                "AAAA".to_string(),
                "BBBB".to_string(),
                "CCCC".to_string(),
                "DDDD".to_string(),
            ]),
            recovery_key: "test".to_string(),
            generated_at: "2026-07-19T00:00:00Z".to_string(),
        };
        assert!(valid.validate_split_key().is_ok());
    }

    #[test]
    fn split_key_parts_none_is_valid() {
        use shared::KeyDisplayBundle;
        let bundle = KeyDisplayBundle {
            split_key_parts: None,
            recovery_key: "test".to_string(),
            generated_at: "2026-07-19T00:00:00Z".to_string(),
        };
        assert!(bundle.validate_split_key().is_ok());
    }

    #[test]
    fn split_key_parts_rejects_wrong_count() {
        use shared::KeyDisplayBundle;

        let invalid = KeyDisplayBundle {
            split_key_parts: Some(vec!["AAAA".to_string(), "BBBB".to_string()]),
            recovery_key: "test".to_string(),
            generated_at: "2026-07-19T00:00:00Z".to_string(),
        };
        assert!(invalid.validate_split_key().is_err());
    }

    #[test]
    fn split_key_parts_rejects_wrong_part_length() {
        use shared::KeyDisplayBundle;

        let invalid = KeyDisplayBundle {
            split_key_parts: Some(vec![
                "AAA".to_string(),  // 3 chars, not 4
                "BBBB".to_string(),
                "CCCC".to_string(),
                "DDDD".to_string(),
            ]),
            recovery_key: "test".to_string(),
            generated_at: "2026-07-19T00:00:00Z".to_string(),
        };
        assert!(invalid.validate_split_key().is_err());
    }

    #[test]
    fn split_key_parts_rejects_5_parts() {
        use shared::KeyDisplayBundle;
        let invalid = KeyDisplayBundle {
            split_key_parts: Some(vec![
                "AAAA".to_string(),
                "BBBB".to_string(),
                "CCCC".to_string(),
                "DDDD".to_string(),
                "EEEE".to_string(),
            ]),
            recovery_key: "test".to_string(),
            generated_at: "2026-07-19T00:00:00Z".to_string(),
        };
        assert!(invalid.validate_split_key().is_err());
    }

    // ─── IPC Nonce tests ──────────────────────────────────────────────────────

    #[test]
    fn ipc_envelope_serializes_correctly() {
        use shared::IpcEnvelope;

        let env = IpcEnvelope::new(1, "watch_folder", serde_json::json!({ "path": "/tmp/test" }));
        let val = serde_json::to_value(&env).unwrap();
        assert_eq!(val["nonce"], 1);
        assert_eq!(val["command"], "watch_folder");
        assert_eq!(val["payload"]["path"], "/tmp/test");
    }

    #[test]
    fn ipc_envelope_nonce_is_preserved() {
        use shared::IpcEnvelope;
        let env = IpcEnvelope::new(42, "service_status", serde_json::json!({}));
        assert_eq!(env.nonce, 42);
        assert_eq!(env.command, "service_status");
    }

    #[test]
    fn settings_default_values_match_spec() {
        use shared::Settings;
        let s = Settings::default();
        assert_eq!(s.session_timeout_minutes, 15, "Default timeout must be 15 min per spec");
        assert_eq!(s.max_login_attempts, 3, "Default max attempts must be 3 per spec");
        assert_eq!(s.lockout_duration_secs, 900, "Default lockout must be 15 min per spec");
        assert!(s.realtime_integrity_alerts, "Realtime integrity alerts must be enabled by default");
    }

    // ─── ProtectedItemState tests ─────────────────────────────────────────────

    #[test]
    fn protected_item_state_serializes() {
        use shared::ProtectedItemState;

        let states = [
            ProtectedItemState::Idle,
            ProtectedItemState::Protecting,
            ProtectedItemState::Protected,
            ProtectedItemState::Unprotecting,
            ProtectedItemState::CrashRecoveryPending,
        ];

        for state in &states {
            let serialized = serde_json::to_string(state).expect("Failed to serialize state");
            let deserialized: ProtectedItemState = serde_json::from_str(&serialized)
                .expect("Failed to deserialize state");
            assert_eq!(state, &deserialized);
        }
    }

    // ─── FortiChainError deserialization roundtrip ───────────────────────────

    #[test]
    fn error_roundtrip_auth_invalid_credentials() {
        let err = FortiChainError::AuthInvalidCredentials;
        let json = serde_json::to_string(&err).unwrap();
        let decoded: FortiChainError = serde_json::from_str(&json).unwrap();
        assert_eq!(wire_code(&decoded), "AUTH_INVALID_CREDENTIALS");
    }

    #[test]
    fn error_roundtrip_validation_failed() {
        let err = FortiChainError::ValidationFailed {
            field: "username".into(),
            reason: "cannot be empty".into(),
        };
        let json = serde_json::to_string(&err).unwrap();
        let decoded: FortiChainError = serde_json::from_str(&json).unwrap();
        assert_eq!(wire_code(&decoded), "VALIDATION_FAILED");
        // Ensure detail is preserved
        let val: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert!(val["detail"].is_object());
        assert_eq!(val["detail"]["field"], "username");
    }

    #[test]
    fn error_roundtrip_auth_account_locked() {
        let err = FortiChainError::AuthAccountLocked { retry_after_secs: 600 };
        let json = serde_json::to_string(&err).unwrap();
        let decoded: FortiChainError = serde_json::from_str(&json).unwrap();
        assert_eq!(wire_code(&decoded), "AUTH_ACCOUNT_LOCKED");
        let val: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(val["detail"]["retry_after_secs"], 600);
    }

    #[test]
    fn error_roundtrip_folder_state_conflict() {
        let err = FortiChainError::FolderStateConflict {
            current_state: "Protected".into(),
        };
        let json = serde_json::to_string(&err).unwrap();
        let val: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(val["code"], "FOLDER_STATE_CONFLICT");
        assert_eq!(val["detail"]["current_state"], "Protected");
    }

    // ─── AuditFilter tests ───────────────────────────────────────────────────

    #[test]
    fn audit_filter_serializes_with_none_fields() {
        use shared::AuditFilter;
        let filter = AuditFilter {
            since: None,
            until: None,
            action_contains: None,
            limit: None,
            after_id: None,
        };
        let json = serde_json::to_string(&filter).unwrap();
        assert!(json.contains("null") || !json.contains("\"since\":\""), 
            "None fields should serialize as null or be omitted");
    }

    #[test]
    fn audit_filter_with_limit() {
        use shared::AuditFilter;
        let filter = AuditFilter {
            since: None,
            until: None,
            action_contains: Some("LOGIN".to_string()),
            limit: Some(50),
            after_id: Some(100),
        };
        let val = serde_json::to_value(&filter).unwrap();
        assert_eq!(val["limit"], 50);
        assert_eq!(val["after_id"], 100);
        assert_eq!(val["action_contains"], "LOGIN");
    }

    // ─── IPC Command constants ───────────────────────────────────────────────

    #[test]
    fn ipc_command_constants_are_correct() {
        use shared::ipc_commands;
        assert_eq!(ipc_commands::WATCH_FOLDER, "watch_folder");
        assert_eq!(ipc_commands::UNWATCH_FOLDER, "unwatch_folder");
        assert_eq!(ipc_commands::ANCHOR_AUDIT_ENTRY, "anchor_audit_entry");
        assert_eq!(ipc_commands::INTEGRITY_ALERT, "integrity_alert");
        assert_eq!(ipc_commands::SERVICE_STATUS, "service_status");
    }
}

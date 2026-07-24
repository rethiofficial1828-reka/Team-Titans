-- 001_initial.sql — FortiChain initial database schema
-- All tables created only if they don't already exist (idempotent migration)

-- Schema version tracker
CREATE TABLE IF NOT EXISTS schema_version (
    version INTEGER NOT NULL,
    applied_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Administrators / users
CREATE TABLE IF NOT EXISTS users (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    username    TEXT NOT NULL UNIQUE,
    -- Argon2id PHC string: "$argon2id$v=19$..."
    password_hash TEXT NOT NULL,
    role        TEXT NOT NULL CHECK(role IN (
                    'SuperAdmin','Admin','Investigator',
                    'Auditor','ReadOnly','RecoveryAdmin')),
    created_at  TEXT NOT NULL DEFAULT (datetime('now')),
    is_active   INTEGER NOT NULL DEFAULT 1
);

-- Admin split-key material (16 chars split into 4×4)
-- Stored as Argon2id hash — the actual key parts are never stored in plaintext
CREATE TABLE IF NOT EXISTS admin_keys (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id     INTEGER NOT NULL REFERENCES users(id),
    -- Argon2id hash of the concatenated key (all 4 parts joined)
    key_hash    TEXT NOT NULL,
    created_at  TEXT NOT NULL DEFAULT (datetime('now')),
    is_active   INTEGER NOT NULL DEFAULT 1
);

-- Recovery keys — AES-256-GCM encrypted, stored with nonce (fixes issue #6)
-- Format: hex(nonce || ciphertext) where nonce is 12 bytes
CREATE TABLE IF NOT EXISTS recovery_keys (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id         INTEGER NOT NULL REFERENCES users(id),
    encrypted_key   TEXT NOT NULL,  -- hex(nonce||ciphertext)
    generated_at    TEXT NOT NULL DEFAULT (datetime('now')),
    is_active       INTEGER NOT NULL DEFAULT 1
);

-- Active sessions — NOT stored here; sessions are in-memory only.
-- This table tracks lockout state only.
CREATE TABLE IF NOT EXISTS login_attempts (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    username    TEXT NOT NULL,
    success     INTEGER NOT NULL DEFAULT 0,
    attempted_at TEXT NOT NULL DEFAULT (datetime('now')),
    ip_context  TEXT  -- null for desktop app (no network)
);

-- Protected folders/drives
CREATE TABLE IF NOT EXISTS protected_items (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    path            TEXT NOT NULL UNIQUE,
    state           TEXT NOT NULL DEFAULT 'Idle'
                    CHECK(state IN ('Idle','Protecting','Protected',
                                    'Unprotecting','CrashRecoveryPending', 'ReadOnly')),
    protected_at    TEXT,
    protected_by    INTEGER REFERENCES users(id),
    -- Crash recovery: tracks how many files have been processed
    files_processed INTEGER NOT NULL DEFAULT 0,
    files_total     INTEGER,
    -- Per-item DEK, wrapped with master key via AES-256-GCM
    -- hex(nonce||ciphertext)
    wrapped_dek     TEXT,
    created_at      TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Immutable audit log with hash chain
-- Chain formula: HMAC-SHA3-512(chain_key, prev_hash || entry_json)
CREATE TABLE IF NOT EXISTS audit_log (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    timestamp   TEXT NOT NULL DEFAULT (datetime('now')),
    user_id     INTEGER REFERENCES users(id),
    action      TEXT NOT NULL,
    detail      TEXT,
    -- The chain hash for this entry — verified by verify_audit_chain command
    chain_hash  TEXT NOT NULL,
    -- Genesis marker: chain_hash of the very first entry is computed with prev_hash="genesis"
    is_genesis  INTEGER NOT NULL DEFAULT 0
);

-- Security policies / settings
CREATE TABLE IF NOT EXISTS settings (
    id                          INTEGER PRIMARY KEY CHECK(id = 1),
    session_timeout_minutes     INTEGER NOT NULL DEFAULT 15,
    max_login_attempts          INTEGER NOT NULL DEFAULT 3,
    lockout_duration_secs       INTEGER NOT NULL DEFAULT 900,
    realtime_integrity_alerts   INTEGER NOT NULL DEFAULT 1,
    updated_at                  TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Insert default settings row (upsert — safe to run multiple times)
INSERT OR IGNORE INTO settings (id) VALUES (1);

-- Insert initial schema version
INSERT OR IGNORE INTO schema_version (version) VALUES (1);

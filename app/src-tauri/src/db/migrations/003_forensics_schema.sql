-- 003_forensics_schema.sql — Attack Forensics & Threat Intelligence Center Schema

CREATE TABLE IF NOT EXISTS incidents (
    incident_id       TEXT PRIMARY KEY,
    created_at        INTEGER NOT NULL,
    updated_at        INTEGER NOT NULL,
    attack_type       TEXT NOT NULL,
    severity          TEXT NOT NULL CHECK (severity IN ('LOW','MEDIUM','HIGH','CRITICAL')),
    risk_score        INTEGER NOT NULL CHECK (risk_score BETWEEN 0 AND 100),
    status            TEXT NOT NULL CHECK (status IN ('OPEN','BLOCKED','RESOLVED','ALLOWED')),
    username          TEXT,
    computer_name     TEXT,
    target_folder     TEXT,
    event_count       INTEGER NOT NULL DEFAULT 1,
    first_seen        INTEGER NOT NULL,
    last_seen         INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS attack_logs (
    log_id            INTEGER PRIMARY KEY AUTOINCREMENT,
    incident_id       TEXT NOT NULL REFERENCES incidents(incident_id) ON DELETE CASCADE,
    timestamp         INTEGER NOT NULL,
    attack_type       TEXT NOT NULL,
    severity          TEXT NOT NULL,
    risk_score        INTEGER NOT NULL,
    username          TEXT,
    computer_name     TEXT,
    process_name      TEXT,
    process_id        INTEGER,
    executable_path   TEXT,
    target_folder     TEXT,
    target_file       TEXT,
    action_taken      TEXT,
    status            TEXT NOT NULL,
    sha3_hash         TEXT NOT NULL,
    prev_hash         TEXT NOT NULL,
    remarks           TEXT
);

CREATE TABLE IF NOT EXISTS timeline (
    id                INTEGER PRIMARY KEY AUTOINCREMENT,
    incident_id       TEXT NOT NULL REFERENCES incidents(incident_id) ON DELETE CASCADE,
    step_order        INTEGER NOT NULL,
    label             TEXT NOT NULL,
    timestamp         INTEGER NOT NULL,
    detail            TEXT
);

CREATE TABLE IF NOT EXISTS recommendations (
    id                INTEGER PRIMARY KEY AUTOINCREMENT,
    incident_id       TEXT NOT NULL REFERENCES incidents(incident_id) ON DELETE CASCADE,
    recommendation    TEXT NOT NULL,
    priority          TEXT NOT NULL,
    applied           INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS statistics (
    stat_date         TEXT PRIMARY KEY,
    total_attacks     INTEGER NOT NULL DEFAULT 0,
    critical_attacks  INTEGER NOT NULL DEFAULT 0,
    blocked_attacks   INTEGER NOT NULL DEFAULT 0,
    allowed_attacks   INTEGER NOT NULL DEFAULT 0,
    avg_response_ms   INTEGER NOT NULL DEFAULT 0
);

CREATE INDEX IF NOT EXISTS idx_logs_incident   ON attack_logs(incident_id);
CREATE INDEX IF NOT EXISTS idx_logs_timestamp  ON attack_logs(timestamp);
CREATE INDEX IF NOT EXISTS idx_logs_severity   ON attack_logs(severity);
CREATE INDEX IF NOT EXISTS idx_logs_user       ON attack_logs(username);
CREATE INDEX IF NOT EXISTS idx_incidents_status ON incidents(status);
CREATE INDEX IF NOT EXISTS idx_incidents_severity ON incidents(severity);

-- 002_update_check.sql

PRAGMA foreign_keys=OFF;

CREATE TABLE protected_items_new (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    path            TEXT NOT NULL UNIQUE,
    state           TEXT NOT NULL DEFAULT 'Idle'
                    CHECK(state IN ('Idle','Protecting','Protected',
                                    'Unprotecting','CrashRecoveryPending','ReadOnly')),
    protected_at    TEXT,
    protected_by    INTEGER REFERENCES users(id),
    files_processed INTEGER NOT NULL DEFAULT 0,
    files_total     INTEGER,
    wrapped_dek     TEXT,
    created_at      TEXT NOT NULL DEFAULT (datetime('now'))
);

INSERT INTO protected_items_new SELECT * FROM protected_items;
DROP TABLE protected_items;
ALTER TABLE protected_items_new RENAME TO protected_items;

PRAGMA foreign_keys=ON;

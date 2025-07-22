CREATE TABLE IF NOT EXISTS games (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    owner_id TEXT NOT NULL,
    voting_system TEXT NOT NULL,
    state TEXT NOT NULL DEFAULT 'Waiting',
    current_story TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);
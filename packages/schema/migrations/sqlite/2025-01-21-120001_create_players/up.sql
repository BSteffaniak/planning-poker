CREATE TABLE IF NOT EXISTS players (
    id TEXT PRIMARY KEY NOT NULL,
    game_id TEXT NOT NULL,
    name TEXT NOT NULL,
    is_observer INTEGER NOT NULL DEFAULT 0,
    joined_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (game_id) REFERENCES games(id) ON DELETE CASCADE
);
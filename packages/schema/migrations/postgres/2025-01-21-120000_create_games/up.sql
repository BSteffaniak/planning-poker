CREATE TABLE IF NOT EXISTS games (
    id VARCHAR(36) PRIMARY KEY NOT NULL,
    name VARCHAR(255) NOT NULL,
    owner_id VARCHAR(36) NOT NULL,
    voting_system VARCHAR(50) NOT NULL,
    state VARCHAR(20) NOT NULL DEFAULT 'Waiting',
    current_story TEXT,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);
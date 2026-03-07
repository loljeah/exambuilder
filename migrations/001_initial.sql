-- Knowledge Gate initial schema

CREATE TABLE IF NOT EXISTS projects (
    id TEXT PRIMARY KEY,
    full_hash TEXT NOT NULL,
    path TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    last_active TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS debt_log (
    id INTEGER PRIMARY KEY,
    project_id TEXT NOT NULL REFERENCES projects(id),
    action TEXT NOT NULL,
    weight INTEGER NOT NULL,
    description TEXT,
    timestamp TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS debt_current (
    project_id TEXT PRIMARY KEY REFERENCES projects(id),
    total INTEGER NOT NULL DEFAULT 0,
    last_updated TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS sprints (
    id INTEGER PRIMARY KEY,
    project_id TEXT NOT NULL REFERENCES projects(id),
    sprint_number INTEGER NOT NULL,
    topic TEXT NOT NULL,
    questions_json TEXT NOT NULL,
    answer_key_json TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'pending',
    best_score INTEGER,
    attempts INTEGER DEFAULT 0,
    xp_available INTEGER NOT NULL,
    xp_earned INTEGER DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    passed_at TEXT,
    UNIQUE(project_id, sprint_number)
);

CREATE TABLE IF NOT EXISTS profile (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    total_xp INTEGER NOT NULL DEFAULT 0,
    level INTEGER NOT NULL DEFAULT 1,
    current_streak INTEGER NOT NULL DEFAULT 0,
    best_streak INTEGER NOT NULL DEFAULT 0,
    sprints_passed INTEGER NOT NULL DEFAULT 0,
    last_activity TEXT
);

CREATE TABLE IF NOT EXISTS badges (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT NOT NULL,
    icon TEXT,
    rarity TEXT DEFAULT 'common',
    unlocked_at TEXT,
    project_id TEXT REFERENCES projects(id)
);

-- Initialize singleton profile
INSERT OR IGNORE INTO profile (id) VALUES (1);

-- Spaced Repetition Tables (Phase 3)
-- SM-2 algorithm implementation for optimal review scheduling

-- Review items for spaced repetition
CREATE TABLE IF NOT EXISTS review_items (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    project_id TEXT NOT NULL,
    sprint_number INTEGER NOT NULL,
    question_number INTEGER NOT NULL,
    question_text TEXT NOT NULL,
    correct_answer TEXT NOT NULL,
    domain TEXT NOT NULL,

    -- SM-2 algorithm fields
    easiness_factor REAL NOT NULL DEFAULT 2.5,
    repetition_count INTEGER NOT NULL DEFAULT 0,
    interval_days INTEGER NOT NULL DEFAULT 0,
    next_review DATETIME NOT NULL DEFAULT (datetime('now')),
    last_reviewed DATETIME,

    -- Performance tracking
    times_correct INTEGER NOT NULL DEFAULT 0,
    times_wrong INTEGER NOT NULL DEFAULT 0,
    streak INTEGER NOT NULL DEFAULT 0,

    created_at DATETIME NOT NULL DEFAULT (datetime('now')),

    UNIQUE(project_id, sprint_number, question_number)
);

-- Domain catalog for collecting questions across projects
CREATE TABLE IF NOT EXISTS domain_catalog (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    domain TEXT NOT NULL,
    question_id TEXT NOT NULL UNIQUE,
    question_text TEXT NOT NULL,
    correct_answer TEXT NOT NULL,
    source_project TEXT NOT NULL,
    source_sprint INTEGER NOT NULL,
    tier TEXT NOT NULL,
    difficulty TEXT NOT NULL,
    times_seen INTEGER NOT NULL DEFAULT 0,
    times_correct INTEGER NOT NULL DEFAULT 0,
    last_seen DATETIME,
    tags_json TEXT,  -- JSON array of tags
    created_at DATETIME NOT NULL DEFAULT (datetime('now'))
);

-- Index for efficient domain queries
CREATE INDEX IF NOT EXISTS idx_domain_catalog_domain ON domain_catalog(domain);
CREATE INDEX IF NOT EXISTS idx_review_items_next ON review_items(next_review);
CREATE INDEX IF NOT EXISTS idx_review_items_domain ON review_items(domain);

-- Adaptive difficulty tracking (Phase 4)
CREATE TABLE IF NOT EXISTS difficulty_profile (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    domain TEXT NOT NULL UNIQUE,

    -- Performance by tier
    recall_accuracy REAL NOT NULL DEFAULT 0.0,
    comprehension_accuracy REAL NOT NULL DEFAULT 0.0,
    application_accuracy REAL NOT NULL DEFAULT 0.0,
    analysis_accuracy REAL NOT NULL DEFAULT 0.0,

    -- Question counts by tier
    recall_count INTEGER NOT NULL DEFAULT 0,
    comprehension_count INTEGER NOT NULL DEFAULT 0,
    application_count INTEGER NOT NULL DEFAULT 0,
    analysis_count INTEGER NOT NULL DEFAULT 0,

    -- Recommended difficulty level (1-5)
    recommended_level INTEGER NOT NULL DEFAULT 2,

    -- Streak tracking for difficulty adjustment
    consecutive_correct INTEGER NOT NULL DEFAULT 0,
    consecutive_wrong INTEGER NOT NULL DEFAULT 0,

    last_updated DATETIME NOT NULL DEFAULT (datetime('now'))
);

-- Review session history
CREATE TABLE IF NOT EXISTS review_sessions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    started_at DATETIME NOT NULL DEFAULT (datetime('now')),
    ended_at DATETIME,
    items_reviewed INTEGER NOT NULL DEFAULT 0,
    items_correct INTEGER NOT NULL DEFAULT 0,
    domains_covered TEXT,  -- JSON array
    xp_earned INTEGER NOT NULL DEFAULT 0
);

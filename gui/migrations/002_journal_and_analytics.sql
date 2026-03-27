-- Migration 002: Add journal, analytics, and future-ready tables
-- This adds comprehensive tracking for all events, knowledge items, and analytics

-- ============================================================================
-- ACTIVITY JOURNAL - Timestamped log of ALL events
-- ============================================================================
CREATE TABLE journal (
    id INTEGER PRIMARY KEY,
    timestamp TEXT NOT NULL DEFAULT (datetime('now')),
    event_type TEXT NOT NULL,  -- See event types below
    project_id TEXT REFERENCES projects(id),
    sprint_id INTEGER REFERENCES sprints(id),
    data_json TEXT,  -- Event-specific payload
    session_id TEXT  -- Groups events from same daemon session
);

-- Event types:
-- daemon_start, daemon_stop
-- project_activated, project_created
-- exam_imported, exam_updated
-- sprint_started, sprint_completed, sprint_passed, sprint_failed
-- question_answered (correct/incorrect)
-- debt_added, debt_cleared
-- level_up, streak_updated, streak_broken
-- badge_unlocked
-- voice_mode_used

CREATE INDEX idx_journal_timestamp ON journal(timestamp);
CREATE INDEX idx_journal_event_type ON journal(event_type);
CREATE INDEX idx_journal_project ON journal(project_id);
CREATE INDEX idx_journal_session ON journal(session_id);

-- ============================================================================
-- KNOWLEDGE ITEMS - Individual concepts extracted from exams
-- ============================================================================
CREATE TABLE knowledge_items (
    id INTEGER PRIMARY KEY,
    project_id TEXT NOT NULL REFERENCES projects(id),
    sprint_id INTEGER REFERENCES sprints(id),
    question_number INTEGER,

    -- The concept
    concept TEXT NOT NULL,           -- Short name: "Unix sockets", "Go interfaces"
    category TEXT,                   -- "networking", "language", "architecture"
    tier TEXT,                       -- RECALL, COMPREHENSION, APPLICATION, ANALYSIS

    -- Learning status
    status TEXT DEFAULT 'unseen',    -- unseen, learning, mastered
    times_seen INTEGER DEFAULT 0,
    times_correct INTEGER DEFAULT 0,
    times_incorrect INTEGER DEFAULT 0,
    mastery_score REAL DEFAULT 0.0,  -- 0.0 to 1.0, calculated from accuracy + recency

    -- Spaced repetition
    next_review TEXT,                -- When to review next
    ease_factor REAL DEFAULT 2.5,    -- SM-2 algorithm ease factor
    interval_days INTEGER DEFAULT 1,

    -- Timestamps
    first_seen TEXT,
    last_seen TEXT,
    last_correct TEXT,
    mastered_at TEXT,

    UNIQUE(project_id, concept)
);

CREATE INDEX idx_knowledge_project ON knowledge_items(project_id);
CREATE INDEX idx_knowledge_status ON knowledge_items(status);
CREATE INDEX idx_knowledge_review ON knowledge_items(next_review);

-- ============================================================================
-- QUESTION STATS - Per-question accuracy tracking
-- ============================================================================
CREATE TABLE question_stats (
    id INTEGER PRIMARY KEY,
    sprint_id INTEGER NOT NULL REFERENCES sprints(id),
    question_number INTEGER NOT NULL,
    question_hash TEXT,              -- Hash of question text for dedup

    -- Stats
    times_shown INTEGER DEFAULT 0,
    times_correct INTEGER DEFAULT 0,
    times_incorrect INTEGER DEFAULT 0,
    times_skipped INTEGER DEFAULT 0,

    -- Timing
    avg_response_time_ms INTEGER,
    fastest_response_ms INTEGER,
    slowest_response_ms INTEGER,

    -- Common wrong answers
    wrong_answers_json TEXT,         -- {"A": 5, "C": 2} - frequency of wrong choices

    -- Timestamps
    first_shown TEXT,
    last_shown TEXT,
    last_correct TEXT,

    UNIQUE(sprint_id, question_number)
);

CREATE INDEX idx_qstats_sprint ON question_stats(sprint_id);

-- ============================================================================
-- SESSIONS - Track daemon sessions
-- ============================================================================
CREATE TABLE sessions (
    id TEXT PRIMARY KEY,             -- UUID
    started_at TEXT NOT NULL DEFAULT (datetime('now')),
    ended_at TEXT,
    duration_seconds INTEGER,

    -- Session stats
    commands_received INTEGER DEFAULT 0,
    sprints_taken INTEGER DEFAULT 0,
    sprints_passed INTEGER DEFAULT 0,
    xp_earned INTEGER DEFAULT 0,

    -- Environment
    hostname TEXT,
    username TEXT,
    version TEXT
);

-- ============================================================================
-- DAILY STATS - Aggregated daily statistics
-- ============================================================================
CREATE TABLE daily_stats (
    date TEXT PRIMARY KEY,           -- YYYY-MM-DD

    -- Activity
    sessions_count INTEGER DEFAULT 0,
    active_minutes INTEGER DEFAULT 0,
    commands_count INTEGER DEFAULT 0,

    -- Learning
    sprints_attempted INTEGER DEFAULT 0,
    sprints_passed INTEGER DEFAULT 0,
    questions_answered INTEGER DEFAULT 0,
    questions_correct INTEGER DEFAULT 0,

    -- Progress
    xp_earned INTEGER DEFAULT 0,
    debt_added INTEGER DEFAULT 0,
    debt_cleared INTEGER DEFAULT 0,

    -- Streaks
    streak_at_end INTEGER DEFAULT 0,

    -- First/last activity
    first_activity TEXT,
    last_activity TEXT
);

-- ============================================================================
-- WEEKLY GOALS - Optional goal tracking
-- ============================================================================
CREATE TABLE weekly_goals (
    id INTEGER PRIMARY KEY,
    week_start TEXT NOT NULL,        -- YYYY-MM-DD (Monday)

    -- Goals
    target_sprints INTEGER DEFAULT 5,
    target_xp INTEGER DEFAULT 100,
    target_streak INTEGER DEFAULT 3,

    -- Progress
    actual_sprints INTEGER DEFAULT 0,
    actual_xp INTEGER DEFAULT 0,
    max_streak INTEGER DEFAULT 0,

    -- Status
    completed INTEGER DEFAULT 0,
    completed_at TEXT,

    UNIQUE(week_start)
);

-- ============================================================================
-- STUDY NOTES - User notes on concepts/questions
-- ============================================================================
CREATE TABLE study_notes (
    id INTEGER PRIMARY KEY,
    project_id TEXT REFERENCES projects(id),
    sprint_id INTEGER REFERENCES sprints(id),
    question_number INTEGER,
    knowledge_item_id INTEGER REFERENCES knowledge_items(id),

    note TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_notes_project ON study_notes(project_id);
CREATE INDEX idx_notes_knowledge ON study_notes(knowledge_item_id);

-- ============================================================================
-- EXPORT HISTORY - Track exports for sync
-- ============================================================================
CREATE TABLE export_history (
    id INTEGER PRIMARY KEY,
    exported_at TEXT NOT NULL DEFAULT (datetime('now')),
    export_type TEXT NOT NULL,       -- full, incremental, journal_only
    file_path TEXT,
    file_hash TEXT,
    records_count INTEGER,
    size_bytes INTEGER
);

-- ============================================================================
-- SETTINGS - Key-value store for user preferences
-- ============================================================================
CREATE TABLE settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Default settings
INSERT INTO settings (key, value) VALUES
    ('theme', 'system'),
    ('voice_enabled', 'true'),
    ('notifications_enabled', 'true'),
    ('daily_goal_sprints', '2'),
    ('daily_goal_xp', '50'),
    ('spaced_repetition_enabled', 'true'),
    ('auto_export_enabled', 'false'),
    ('export_format', 'json');

-- ============================================================================
-- TAGS - Flexible tagging for projects/sprints/knowledge
-- ============================================================================
CREATE TABLE tags (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    color TEXT DEFAULT '#6b7280',
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE tagged_items (
    tag_id INTEGER NOT NULL REFERENCES tags(id),
    item_type TEXT NOT NULL,         -- project, sprint, knowledge_item
    item_id TEXT NOT NULL,           -- ID of the tagged item
    tagged_at TEXT NOT NULL DEFAULT (datetime('now')),
    PRIMARY KEY (tag_id, item_type, item_id)
);

-- ============================================================================
-- MILESTONES - Track major achievements
-- ============================================================================
CREATE TABLE milestones (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT,
    category TEXT,                   -- xp, streak, sprints, mastery
    threshold INTEGER,               -- Value needed to unlock
    icon TEXT,

    -- Progress
    current_value INTEGER DEFAULT 0,
    unlocked INTEGER DEFAULT 0,
    unlocked_at TEXT,

    -- Display
    display_order INTEGER DEFAULT 0,
    hidden INTEGER DEFAULT 0         -- Hidden until close to unlocking
);

-- Default milestones
INSERT INTO milestones (id, name, description, category, threshold, icon, display_order) VALUES
    ('xp_100', 'First Steps', 'Earn 100 XP', 'xp', 100, '🎯', 1),
    ('xp_500', 'Getting Serious', 'Earn 500 XP', 'xp', 500, '📚', 2),
    ('xp_1000', 'Knowledge Seeker', 'Earn 1000 XP', 'xp', 1000, '🧠', 3),
    ('xp_5000', 'Scholar', 'Earn 5000 XP', 'xp', 5000, '🎓', 4),
    ('streak_3', 'On a Roll', '3 sprint streak', 'streak', 3, '🔥', 10),
    ('streak_7', 'Weekly Warrior', '7 sprint streak', 'streak', 7, '⚡', 11),
    ('streak_30', 'Monthly Master', '30 sprint streak', 'streak', 30, '💫', 12),
    ('sprints_10', 'Sprint Starter', 'Pass 10 sprints', 'sprints', 10, '🏃', 20),
    ('sprints_50', 'Sprint Champion', 'Pass 50 sprints', 'sprints', 50, '🏆', 21),
    ('sprints_100', 'Century Club', 'Pass 100 sprints', 'sprints', 100, '💯', 22),
    ('mastery_10', 'Concept Collector', 'Master 10 concepts', 'mastery', 10, '✨', 30),
    ('mastery_50', 'Knowledge Base', 'Master 50 concepts', 'mastery', 50, '📖', 31),
    ('perfect_sprint', 'Perfectionist', 'Score 100% on a sprint', 'special', 1, '💎', 40),
    ('voice_first', 'Voice Activated', 'Complete a sprint in voice mode', 'special', 1, '🎙️', 41);

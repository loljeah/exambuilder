-- Migration 004: Domain-specific stats, achievements, and leveling
-- Supports multi-domain knowledge tracking with auto-generated achievements

-- ============================================================================
-- DOMAINS - Knowledge domains per project
-- ============================================================================
CREATE TABLE domains (
    id TEXT PRIMARY KEY,                    -- project_id + "_" + domain_id
    project_id TEXT NOT NULL REFERENCES projects(id),
    domain_id TEXT NOT NULL,                -- e.g., "architecture", "security"
    name TEXT NOT NULL,                     -- Display name
    description TEXT,
    color TEXT DEFAULT '#6B7280',           -- Hex color for UI
    icon TEXT DEFAULT '📚',                 -- Emoji icon
    total_xp INT DEFAULT 0,                 -- Total XP available
    earned_xp INT DEFAULT 0,                -- XP earned so far
    level INT DEFAULT 1,                    -- Current level in this domain
    sprints_total INT DEFAULT 0,
    sprints_passed INT DEFAULT 0,
    sprints_perfect INT DEFAULT 0,          -- 100% score
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(project_id, domain_id)
);

CREATE INDEX idx_domains_project ON domains(project_id);

-- ============================================================================
-- DOMAIN_LEVELS - Level definitions per domain
-- ============================================================================
CREATE TABLE domain_levels (
    id INTEGER PRIMARY KEY,
    project_id TEXT NOT NULL REFERENCES projects(id),
    domain_id TEXT NOT NULL,
    level INT NOT NULL,
    xp_threshold INT NOT NULL,              -- XP needed for this level
    title TEXT NOT NULL,                    -- e.g., "Novice", "Expert"
    UNIQUE(project_id, domain_id, level)
);

-- ============================================================================
-- DOMAIN_ACHIEVEMENTS - Auto-generated achievements per domain
-- ============================================================================
CREATE TABLE domain_achievements (
    id TEXT PRIMARY KEY,                    -- project_id + "_" + achievement_id
    project_id TEXT NOT NULL REFERENCES projects(id),
    domain_id TEXT NOT NULL,
    name TEXT NOT NULL,
    description TEXT NOT NULL,
    condition TEXT NOT NULL,                -- e.g., "domain_sprints_passed >= 1"
    xp_reward INT DEFAULT 0,
    icon TEXT DEFAULT '🏆',
    unlocked INT DEFAULT 0,
    unlocked_at TEXT,
    UNIQUE(project_id, domain_id, name)
);

CREATE INDEX idx_domain_achievements_project ON domain_achievements(project_id);
CREATE INDEX idx_domain_achievements_unlocked ON domain_achievements(unlocked);

-- ============================================================================
-- SUBDOMAINS - Optional finer categorization
-- ============================================================================
CREATE TABLE subdomains (
    id TEXT PRIMARY KEY,                    -- project_id + "_" + domain_id + "_" + subdomain_id
    project_id TEXT NOT NULL REFERENCES projects(id),
    domain_id TEXT NOT NULL,
    subdomain_id TEXT NOT NULL,
    name TEXT NOT NULL,
    total_xp INT DEFAULT 0,
    earned_xp INT DEFAULT 0,
    UNIQUE(project_id, domain_id, subdomain_id)
);

-- ============================================================================
-- DOMAIN_STATS - Aggregate stats view helper
-- ============================================================================
CREATE TABLE domain_stats (
    id INTEGER PRIMARY KEY,
    project_id TEXT NOT NULL REFERENCES projects(id),
    domain_id TEXT NOT NULL,

    -- Accuracy
    questions_total INT DEFAULT 0,
    questions_correct INT DEFAULT 0,

    -- Time tracking
    time_spent_seconds INT DEFAULT 0,

    -- Streak within domain
    current_streak INT DEFAULT 0,
    best_streak INT DEFAULT 0,

    -- Last activity
    last_activity TEXT,

    UNIQUE(project_id, domain_id)
);

-- ============================================================================
-- Update sprints table to include domain reference
-- ============================================================================
ALTER TABLE sprints ADD COLUMN domain_id TEXT;
ALTER TABLE sprints ADD COLUMN subdomain_id TEXT;

-- ============================================================================
-- CONTENT_TYPES - Track what type of content each project is
-- ============================================================================
ALTER TABLE projects ADD COLUMN content_type TEXT DEFAULT 'code';
-- Values: code, medical, legal, scientific, technical, study, other

-- ============================================================================
-- Helper view for domain overview
-- ============================================================================
CREATE VIEW domain_overview AS
SELECT
    d.project_id,
    d.domain_id,
    d.name,
    d.icon,
    d.color,
    d.level,
    d.earned_xp,
    d.total_xp,
    CASE WHEN d.total_xp > 0 THEN (d.earned_xp * 100 / d.total_xp) ELSE 0 END as progress_pct,
    d.sprints_passed,
    d.sprints_total,
    d.sprints_perfect,
    ds.questions_correct,
    ds.questions_total,
    CASE WHEN ds.questions_total > 0 THEN (ds.questions_correct * 100 / ds.questions_total) ELSE 0 END as accuracy_pct,
    ds.current_streak,
    ds.best_streak
FROM domains d
LEFT JOIN domain_stats ds ON d.project_id = ds.project_id AND d.domain_id = ds.domain_id;

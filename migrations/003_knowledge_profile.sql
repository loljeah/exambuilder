-- Enhanced Knowledge Profile System
-- Supports KNOWLEDGEID, domains, cross-domain connections, achievements

-- User's knowledge identity
CREATE TABLE IF NOT EXISTS knowledge_identity (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    knowledge_id TEXT NOT NULL UNIQUE,
    display_name TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    last_sync TEXT
);

-- Knowledge domains (auto-extracted from exam contexts)
CREATE TABLE IF NOT EXISTS domains (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    category TEXT NOT NULL,  -- 'lang', 'tech', 'concept', 'tool'
    icon TEXT,
    total_xp INTEGER DEFAULT 0,
    mastery_level INTEGER DEFAULT 0,
    questions_seen INTEGER DEFAULT 0,
    questions_correct INTEGER DEFAULT 0
);

-- Domain connections (inter-domain relationships)
CREATE TABLE IF NOT EXISTS domain_connections (
    id INTEGER PRIMARY KEY,
    domain_a TEXT NOT NULL REFERENCES domains(id),
    domain_b TEXT NOT NULL REFERENCES domains(id),
    strength INTEGER DEFAULT 1,  -- increases when answered correctly in same sprint
    discovered_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(domain_a, domain_b)
);

-- Passed questions archive ("gotta collect 'em all")
CREATE TABLE IF NOT EXISTS collected_questions (
    id INTEGER PRIMARY KEY,
    project_id TEXT NOT NULL REFERENCES projects(id),
    sprint_number INTEGER NOT NULL,
    question_number INTEGER NOT NULL,
    question_text TEXT NOT NULL,
    correct_answer TEXT NOT NULL,
    user_answer TEXT NOT NULL,
    tier TEXT NOT NULL,  -- RECALL, COMPREHENSION, etc.
    xp_earned INTEGER NOT NULL,
    domains_json TEXT,  -- array of domain IDs this question touched
    collected_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(project_id, sprint_number, question_number)
);

-- Dynamic achievements (context-aware)
CREATE TABLE IF NOT EXISTS achievements (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT NOT NULL,
    icon TEXT,
    rarity TEXT DEFAULT 'common',
    category TEXT NOT NULL,  -- 'domain', 'streak', 'collection', 'special'
    requirement_json TEXT,  -- dynamic requirements
    unlocked_at TEXT,
    context_json TEXT  -- what triggered this achievement
);

-- Domain progress tracking per project
CREATE TABLE IF NOT EXISTS project_domains (
    project_id TEXT NOT NULL REFERENCES projects(id),
    domain_id TEXT NOT NULL REFERENCES domains(id),
    xp_earned INTEGER DEFAULT 0,
    questions_correct INTEGER DEFAULT 0,
    PRIMARY KEY(project_id, domain_id)
);

-- User settings (sounds, fast-answer mode, etc.)
CREATE TABLE IF NOT EXISTS settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);

-- Default settings
INSERT OR IGNORE INTO settings (key, value) VALUES
    ('sound_enabled', 'true'),
    ('sound_correct', 'chime'),
    ('sound_wrong', 'buzz'),
    ('sound_levelup', 'fanfare'),
    ('fast_answer_mode', 'true'),
    ('show_timer', 'false');

-- Initialize knowledge identity with UUID
INSERT OR IGNORE INTO knowledge_identity (id, knowledge_id)
VALUES (1, lower(hex(randomblob(16))));

-- Pre-seed common domains
INSERT OR IGNORE INTO domains (id, name, category, icon) VALUES
    ('rust', 'Rust', 'lang', '🦀'),
    ('python', 'Python', 'lang', '🐍'),
    ('bash', 'Bash', 'lang', '🐚'),
    ('nix', 'Nix', 'lang', '❄️'),
    ('javascript', 'JavaScript', 'lang', '🟨'),
    ('typescript', 'TypeScript', 'lang', '🔷'),
    ('docker', 'Docker', 'tool', '🐳'),
    ('git', 'Git', 'tool', '🔀'),
    ('linux', 'Linux', 'tech', '🐧'),
    ('networking', 'Networking', 'concept', '🌐'),
    ('security', 'Security', 'concept', '🔒'),
    ('databases', 'Databases', 'concept', '🗄️'),
    ('api', 'APIs', 'concept', '🔌'),
    ('hardware', 'Hardware', 'tech', '🔧'),
    ('embedded', 'Embedded', 'tech', '📟'),
    ('gpu', 'GPU/Graphics', 'tech', '🎮'),
    ('ai_ml', 'AI/ML', 'concept', '🤖'),
    ('devops', 'DevOps', 'concept', '🔄'),
    ('testing', 'Testing', 'concept', '🧪'),
    ('architecture', 'Architecture', 'concept', '🏗️');

-- Pre-seed dynamic achievements
INSERT OR IGNORE INTO achievements (id, name, description, icon, rarity, category, requirement_json) VALUES
    ('polyglot', 'Polyglot', 'Answer correctly in 3+ programming languages', '🗣️', 'uncommon', 'domain', '{"min_langs": 3}'),
    ('full_stack', 'Full Stack', 'Master both frontend and backend domains', '🥞', 'rare', 'domain', '{"domains": ["api", "databases", "javascript"]}'),
    ('systems_sage', 'Systems Sage', 'Master Linux, Networking, and Security', '🧙', 'rare', 'domain', '{"domains": ["linux", "networking", "security"]}'),
    ('collector_10', 'Collector', 'Collect 10 correct answers', '📚', 'common', 'collection', '{"min_collected": 10}'),
    ('collector_50', 'Hoarder', 'Collect 50 correct answers', '📦', 'uncommon', 'collection', '{"min_collected": 50}'),
    ('collector_100', 'Archivist', 'Collect 100 correct answers', '🏛️', 'rare', 'collection', '{"min_collected": 100}'),
    ('domain_master', 'Domain Master', 'Reach mastery level 5 in any domain', '👑', 'rare', 'domain', '{"mastery_level": 5}'),
    ('bridge_builder', 'Bridge Builder', 'Discover 5 domain connections', '🌉', 'uncommon', 'special', '{"connections": 5}'),
    ('speed_round', 'Speed Demon', 'Pass a sprint in under 60 seconds', '⚡', 'uncommon', 'special', '{"max_seconds": 60}'),
    ('night_owl', 'Night Owl', 'Pass a sprint between midnight and 5am', '🦉', 'common', 'special', '{"hour_range": [0, 5]}'),
    ('early_bird', 'Early Bird', 'Pass a sprint between 5am and 7am', '🐦', 'common', 'special', '{"hour_range": [5, 7]}');

-- Index for fast lookups
CREATE INDEX IF NOT EXISTS idx_collected_project ON collected_questions(project_id);
CREATE INDEX IF NOT EXISTS idx_collected_domains ON collected_questions(domains_json);
CREATE INDEX IF NOT EXISTS idx_domain_conn ON domain_connections(domain_a, domain_b);

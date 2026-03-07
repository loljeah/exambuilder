-- Exam attempt history
CREATE TABLE IF NOT EXISTS exam_attempts (
    id INTEGER PRIMARY KEY,
    project_id TEXT NOT NULL REFERENCES projects(id),
    sprint_number INTEGER NOT NULL,
    score_percent INTEGER NOT NULL,
    passed INTEGER NOT NULL DEFAULT 0,
    xp_earned INTEGER NOT NULL DEFAULT 0,
    timestamp TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Index for history queries
CREATE INDEX IF NOT EXISTS idx_attempts_project ON exam_attempts(project_id);
CREATE INDEX IF NOT EXISTS idx_attempts_time ON exam_attempts(timestamp DESC);

-- Pre-populate badge definitions
INSERT OR IGNORE INTO badges (id, name, description, rarity) VALUES
    ('first_sprint', 'First Sprint', 'Pass your first sprint', 'common'),
    ('streak_3', 'On Fire', '3 sprint streak', 'common'),
    ('streak_5', 'Blazing', '5 sprint streak', 'uncommon'),
    ('streak_10', 'Unstoppable', '10 sprint streak', 'rare'),
    ('level_2', 'Config Wrangler', 'Reach level 2', 'common'),
    ('level_3', 'System Operator', 'Reach level 3', 'uncommon'),
    ('level_5', 'Infra Architect', 'Reach level 5', 'rare'),
    ('perfect', 'Perfect Score', '100% on a sprint', 'uncommon'),
    ('project_clear', 'Gate Cleared', 'Pass all sprints in a project', 'uncommon'),
    ('comeback', 'Comeback Kid', 'Pass after 2+ failed attempts', 'common'),
    ('xp_100', 'Century', 'Earn 100 XP total', 'common'),
    ('xp_500', 'Half K', 'Earn 500 XP total', 'uncommon'),
    ('xp_1000', 'Grand Master', 'Earn 1000 XP total', 'rare');

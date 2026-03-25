-- 005_hint_tokens_and_llm.sql
-- Hint token economy + LLM generation support

-- Hint token balance (single-row, like wallet)
CREATE TABLE IF NOT EXISTS hint_tokens (
  id TEXT PRIMARY KEY DEFAULT 'default',
  tokens INTEGER NOT NULL DEFAULT 0,
  lifetime_tokens INTEGER NOT NULL DEFAULT 0
);
INSERT OR IGNORE INTO hint_tokens (id) VALUES ('default');

-- Track which question hints were used on (prevents double-spend)
CREATE TABLE IF NOT EXISTS hint_usage (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  project_id TEXT NOT NULL,
  sprint_number INTEGER NOT NULL,
  question_number INTEGER NOT NULL,
  used_at TEXT NOT NULL DEFAULT (datetime('now')),
  UNIQUE(project_id, sprint_number, question_number)
);
CREATE INDEX IF NOT EXISTS idx_hint_usage_sprint ON hint_usage(project_id, sprint_number);

-- LLM generation audit trail
CREATE TABLE IF NOT EXISTS llm_generations (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  project_id TEXT NOT NULL,
  domain_id TEXT NOT NULL,
  generation_type TEXT NOT NULL,
  model_used TEXT NOT NULL,
  raw_output TEXT,
  sprint_ids TEXT,
  coins_spent INTEGER NOT NULL DEFAULT 0,
  created_at TEXT NOT NULL DEFAULT (datetime('now')),
  status TEXT NOT NULL DEFAULT 'generating'
);
CREATE INDEX IF NOT EXISTS idx_llm_gen_project ON llm_generations(project_id);
CREATE INDEX IF NOT EXISTS idx_llm_gen_status ON llm_generations(status);

-- First-free tracking per domain per project
CREATE TABLE IF NOT EXISTS domain_generation_tracking (
  project_id TEXT NOT NULL,
  domain_id TEXT NOT NULL,
  first_free_used INTEGER NOT NULL DEFAULT 0,
  total_generations INTEGER NOT NULL DEFAULT 0,
  PRIMARY KEY(project_id, domain_id)
);

-- Distinguish imported vs generated sprints
ALTER TABLE sprints ADD COLUMN source TEXT NOT NULL DEFAULT 'imported';

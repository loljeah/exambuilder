-- Enhanced Stats for kgate status display
-- Adds: questions_passed, current_combo, best_combo, total_attempts, accuracy tracking

-- Add new columns to profile
ALTER TABLE profile ADD COLUMN questions_passed INTEGER NOT NULL DEFAULT 0;
ALTER TABLE profile ADD COLUMN questions_attempted INTEGER NOT NULL DEFAULT 0;
ALTER TABLE profile ADD COLUMN current_combo INTEGER NOT NULL DEFAULT 0;
ALTER TABLE profile ADD COLUMN best_combo INTEGER NOT NULL DEFAULT 0;
ALTER TABLE profile ADD COLUMN perfect_sprints INTEGER NOT NULL DEFAULT 0;
ALTER TABLE profile ADD COLUMN total_study_seconds INTEGER NOT NULL DEFAULT 0;

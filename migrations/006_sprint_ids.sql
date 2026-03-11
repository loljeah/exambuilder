-- Sprint IDs and source project names
-- Adds deterministic sprint_id for cross-project reference and source_project_name for display

ALTER TABLE sprints ADD COLUMN sprint_id TEXT;
ALTER TABLE sprints ADD COLUMN source_project_name TEXT;
CREATE UNIQUE INDEX IF NOT EXISTS idx_sprints_sprint_id ON sprints(sprint_id) WHERE sprint_id IS NOT NULL;

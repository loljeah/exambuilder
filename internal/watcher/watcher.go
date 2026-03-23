package watcher

import (
	"os"
	"path/filepath"
	"strings"
	"time"

	"github.com/fsnotify/fsnotify"
	"github.com/loljeah/exambuilder/internal/config"
	"github.com/loljeah/exambuilder/internal/db"
	"github.com/loljeah/exambuilder/internal/exam"
	"github.com/loljeah/exambuilder/internal/logging"
)

type Watcher struct {
	cfg      *config.Config
	db       *db.DB
	watcher  *fsnotify.Watcher
	debounce map[string]time.Time
}

func New(cfg *config.Config, database *db.DB) (*Watcher, error) {
	w, err := fsnotify.NewWatcher()
	if err != nil {
		return nil, err
	}

	return &Watcher{
		cfg:      cfg,
		db:       database,
		watcher:  w,
		debounce: make(map[string]time.Time),
	}, nil
}

func (w *Watcher) Start() error {
	// Watch projects root for exam files
	root := w.cfg.General.ProjectsRoot

	// Walk and add all directories
	err := filepath.Walk(root, func(path string, info os.FileInfo, err error) error {
		if err != nil {
			logging.Debug("watcher walk error", "path", path, "error", err)
			return nil // Skip errors
		}
		if info.IsDir() {
			// Skip hidden directories and common non-project dirs
			name := info.Name()
			if strings.HasPrefix(name, ".") || name == "node_modules" || name == "target" || name == "__pycache__" {
				return filepath.SkipDir
			}
			w.watcher.Add(path)
		}
		return nil
	})
	if err != nil {
		return err
	}

	logging.Info("watching for exam files", "root", root)

	go w.watch()
	return nil
}

func (w *Watcher) Stop() {
	w.watcher.Close()
}

func (w *Watcher) watch() {
	for {
		select {
		case event, ok := <-w.watcher.Events:
			if !ok {
				return
			}
			w.handleEvent(event)

		case err, ok := <-w.watcher.Errors:
			if !ok {
				return
			}
			logging.Error("watcher error", "error", err)
		}
	}
}

func (w *Watcher) handleEvent(event fsnotify.Event) {
	// Only care about writes/creates
	if event.Op&(fsnotify.Write|fsnotify.Create) == 0 {
		return
	}

	name := filepath.Base(event.Name)

	// Only process exam files
	if !strings.HasPrefix(name, "exam_") || !strings.HasSuffix(name, ".md") {
		return
	}

	// Debounce (100ms)
	now := time.Now()
	if last, ok := w.debounce[event.Name]; ok && now.Sub(last) < 100*time.Millisecond {
		return
	}
	w.debounce[event.Name] = now

	// Clean old debounce entries (>1 minute old)
	for k, v := range w.debounce {
		if now.Sub(v) > time.Minute {
			delete(w.debounce, k)
		}
	}

	logging.Info("exam file changed", "path", event.Name)

	// Parse and import
	w.importExamFile(event.Name)
}

func (w *Watcher) importExamFile(path string) {
	content, err := os.ReadFile(path)
	if err != nil {
		logging.Error("read exam file failed", "path", path, "error", err)
		return
	}

	sprints, err := exam.ParseExamFile(string(content))
	if err != nil {
		logging.Error("parse exam file failed", "path", path, "error", err)
		return
	}

	if len(sprints) == 0 {
		logging.Debug("no sprints found in file", "path", path)
		return
	}

	// Determine project from file path
	projectPath := filepath.Dir(path)
	project, err := w.db.GetOrCreateProject(projectPath)
	if err != nil {
		logging.Error("get project failed", "path", projectPath, "error", err)
		return
	}

	// Import sprints
	imported := 0
	for _, ps := range sprints {
		dbSprint, err := ps.ToDBSprint(project.ID)
		if err != nil {
			logging.Warn("convert sprint failed", "sprint", ps.Number, "error", err)
			continue
		}

		if err := w.db.UpsertSprint(dbSprint); err != nil {
			logging.Warn("upsert sprint failed", "sprint", ps.Number, "error", err)
			continue
		}

		imported++
		logging.Debug("imported sprint", "sprint", ps.Number, "topic", ps.Topic, "questions", len(ps.Questions))
	}

	logging.Info("auto-imported exam file", "path", path, "project", project.Name, "sprints", imported)
}

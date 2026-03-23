package watcher

import (
	"log"
	"os"
	"path/filepath"
	"strings"
	"time"

	"github.com/fsnotify/fsnotify"
	"github.com/loljeah/exambuilder/internal/config"
	"github.com/loljeah/exambuilder/internal/db"
	"github.com/loljeah/exambuilder/internal/exam"
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

	log.Printf("watching %s for exam files", root)

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
			log.Printf("watcher error: %v", err)
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

	log.Printf("exam file changed: %s", event.Name)

	// Parse and import
	w.importExamFile(event.Name)
}

func (w *Watcher) importExamFile(path string) {
	content, err := os.ReadFile(path)
	if err != nil {
		log.Printf("read exam file: %v", err)
		return
	}

	sprints, err := exam.ParseExamFile(string(content))
	if err != nil {
		log.Printf("parse exam file: %v", err)
		return
	}

	if len(sprints) == 0 {
		return
	}

	// Determine project from file path
	projectPath := filepath.Dir(path)
	project, err := w.db.GetOrCreateProject(projectPath)
	if err != nil {
		log.Printf("get project: %v", err)
		return
	}

	// Import sprints
	for _, ps := range sprints {
		dbSprint, err := ps.ToDBSprint(project.ID)
		if err != nil {
			log.Printf("convert sprint: %v", err)
			continue
		}

		if err := w.db.UpsertSprint(dbSprint); err != nil {
			log.Printf("upsert sprint: %v", err)
			continue
		}

		log.Printf("imported sprint %d: %s (%d questions)",
			ps.Number, ps.Topic, len(ps.Questions))
	}
}

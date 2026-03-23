package db

import (
	"database/sql"
	"fmt"
	"io/fs"
	"os"
	"path/filepath"
	"sort"
	"strings"

	_ "github.com/mattn/go-sqlite3"
)

type DB struct {
	*sql.DB
}

func Open(path string, migrationsDir string) (*DB, error) {
	if err := os.MkdirAll(filepath.Dir(path), 0755); err != nil {
		return nil, err
	}

	db, err := sql.Open("sqlite3", path+"?_journal_mode=WAL&_foreign_keys=on")
	if err != nil {
		return nil, err
	}

	d := &DB{db}
	if err := d.migrate(migrationsDir); err != nil {
		db.Close()
		return nil, err
	}

	return d, nil
}

func (d *DB) migrate(migrationsDir string) error {
	// Create migrations tracking table
	_, err := d.Exec(`
		CREATE TABLE IF NOT EXISTS migrations (
			id INTEGER PRIMARY KEY,
			name TEXT NOT NULL UNIQUE,
			applied_at TEXT NOT NULL DEFAULT (datetime('now'))
		)
	`)
	if err != nil {
		return err
	}

	// Get applied migrations
	applied := make(map[string]bool)
	rows, err := d.Query("SELECT name FROM migrations")
	if err != nil {
		return err
	}
	defer rows.Close()

	for rows.Next() {
		var name string
		if err := rows.Scan(&name); err != nil {
			return err
		}
		applied[name] = true
	}

	// Read migration files from directory
	entries, err := os.ReadDir(migrationsDir)
	if err != nil {
		// No migrations directory = skip
		if os.IsNotExist(err) {
			return nil
		}
		return err
	}

	// Sort by name
	var names []string
	for _, e := range entries {
		if strings.HasSuffix(e.Name(), ".sql") {
			names = append(names, e.Name())
		}
	}
	sort.Strings(names)

	// Apply pending migrations
	for _, name := range names {
		if applied[name] {
			continue
		}

		content, err := os.ReadFile(filepath.Join(migrationsDir, name))
		if err != nil {
			return err
		}

		tx, err := d.Begin()
		if err != nil {
			return err
		}

		if _, err := tx.Exec(string(content)); err != nil {
			tx.Rollback()
			return fmt.Errorf("migration %s: %w", name, err)
		}

		if _, err := tx.Exec("INSERT INTO migrations (name) VALUES (?)", name); err != nil {
			tx.Rollback()
			return err
		}

		if err := tx.Commit(); err != nil {
			return err
		}
	}

	return nil
}

// OpenWithEmbeddedMigrations opens DB with migrations from an embed.FS
func OpenWithEmbeddedMigrations(path string, migrationsFS fs.FS, subdir string) (*DB, error) {
	if err := os.MkdirAll(filepath.Dir(path), 0755); err != nil {
		return nil, err
	}

	db, err := sql.Open("sqlite3", path+"?_journal_mode=WAL&_foreign_keys=on")
	if err != nil {
		return nil, err
	}

	d := &DB{db}
	if err := d.migrateFromFS(migrationsFS, subdir); err != nil {
		db.Close()
		return nil, err
	}

	return d, nil
}

func (d *DB) migrateFromFS(migrationsFS fs.FS, subdir string) error {
	// Create migrations tracking table
	_, err := d.Exec(`
		CREATE TABLE IF NOT EXISTS migrations (
			id INTEGER PRIMARY KEY,
			name TEXT NOT NULL UNIQUE,
			applied_at TEXT NOT NULL DEFAULT (datetime('now'))
		)
	`)
	if err != nil {
		return err
	}

	// Get applied migrations
	applied := make(map[string]bool)
	rows, err := d.Query("SELECT name FROM migrations")
	if err != nil {
		return err
	}
	defer rows.Close()

	for rows.Next() {
		var name string
		if err := rows.Scan(&name); err != nil {
			return err
		}
		applied[name] = true
	}

	// Read migration files
	entries, err := fs.ReadDir(migrationsFS, subdir)
	if err != nil {
		return err
	}

	// Sort by name
	var names []string
	for _, e := range entries {
		if strings.HasSuffix(e.Name(), ".sql") {
			names = append(names, e.Name())
		}
	}
	sort.Strings(names)

	// Apply pending migrations
	for _, name := range names {
		if applied[name] {
			continue
		}

		content, err := fs.ReadFile(migrationsFS, filepath.Join(subdir, name))
		if err != nil {
			return err
		}

		tx, err := d.Begin()
		if err != nil {
			return err
		}

		if _, err := tx.Exec(string(content)); err != nil {
			tx.Rollback()
			return fmt.Errorf("migration %s: %w", name, err)
		}

		if _, err := tx.Exec("INSERT INTO migrations (name) VALUES (?)", name); err != nil {
			tx.Rollback()
			return err
		}

		if err := tx.Commit(); err != nil {
			return err
		}
	}

	return nil
}

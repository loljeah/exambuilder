package main

import (
	"embed"
	"flag"
	"os"
	"os/signal"
	"syscall"

	"github.com/loljeah/exambuilder/internal/config"
	"github.com/loljeah/exambuilder/internal/daemon"
	"github.com/loljeah/exambuilder/internal/db"
	"github.com/loljeah/exambuilder/internal/logging"
	"github.com/loljeah/exambuilder/internal/tray"
	"github.com/loljeah/exambuilder/internal/watcher"
)

//go:embed migrations/*.sql
var migrationsFS embed.FS

const version = "1.0.0"

func main() {
	verbose := flag.Bool("v", false, "verbose logging")
	noTray := flag.Bool("no-tray", false, "run without system tray")
	flag.Parse()

	// Initialize structured logging
	logging.Init(*verbose)

	// Load config
	cfg, err := config.Load()
	if err != nil {
		logging.Error("failed to load config", "error", err)
		os.Exit(1)
	}

	logging.Info("starting kgate-daemon", "version", version, "config", cfg.ConfigPath())

	// Open database with embedded migrations
	database, err := db.OpenWithEmbeddedMigrations(cfg.DBPath(), migrationsFS, "migrations")
	if err != nil {
		logging.Error("failed to open database", "error", err, "path", cfg.DBPath())
		os.Exit(1)
	}
	defer database.Close()

	// Initialize session for journal tracking
	sessionID, err := database.InitSession(version)
	if err != nil {
		logging.Warn("failed to init session", "error", err)
	} else {
		logging.SetSessionID(sessionID)
		logging.Info("session started")
	}
	defer database.EndSession()

	// Start file watcher
	w, err := watcher.New(cfg, database)
	if err != nil {
		logging.Error("failed to create watcher", "error", err)
		os.Exit(1)
	}
	if err := w.Start(); err != nil {
		logging.Error("failed to start watcher", "error", err)
		os.Exit(1)
	}
	defer w.Stop()

	// Start socket server
	server := daemon.NewServer(cfg, database)

	// Handle shutdown
	sigCh := make(chan os.Signal, 1)
	signal.Notify(sigCh, syscall.SIGINT, syscall.SIGTERM)

	go func() {
		sig := <-sigCh
		logging.Info("shutting down", "signal", sig.String())
		database.EndSession()
		server.Stop()
		os.Exit(0)
	}()

	// Start tray (blocks) or just server
	if *noTray {
		logging.Info("running without tray")
		if err := server.Start(); err != nil {
			logging.Error("server failed", "error", err)
			os.Exit(1)
		}
	} else {
		// Start server in background
		go func() {
			if err := server.Start(); err != nil {
				logging.Error("server failed", "error", err)
				os.Exit(1)
			}
		}()

		// Run tray (blocks)
		t := tray.New(cfg, database)
		t.Run(func() {
			database.EndSession()
			server.Stop()
		})
	}
}

package main

import (
	"embed"
	"flag"
	"log"
	"os"
	"os/signal"
	"syscall"

	"github.com/loljeah/exambuilder/internal/config"
	"github.com/loljeah/exambuilder/internal/daemon"
	"github.com/loljeah/exambuilder/internal/db"
	"github.com/loljeah/exambuilder/internal/tray"
	"github.com/loljeah/exambuilder/internal/watcher"
)

//go:embed migrations/*.sql
var migrationsFS embed.FS

func main() {
	verbose := flag.Bool("v", false, "verbose logging")
	noTray := flag.Bool("no-tray", false, "run without system tray")
	flag.Parse()

	if !*verbose {
		log.SetOutput(os.Stderr)
	}

	// Load config
	cfg, err := config.Load()
	if err != nil {
		log.Fatalf("load config: %v", err)
	}

	// Open database with embedded migrations
	database, err := db.OpenWithEmbeddedMigrations(cfg.DBPath(), migrationsFS, "migrations")
	if err != nil {
		log.Fatalf("open database: %v", err)
	}
	defer database.Close()

	// Start file watcher
	w, err := watcher.New(cfg, database)
	if err != nil {
		log.Fatalf("create watcher: %v", err)
	}
	if err := w.Start(); err != nil {
		log.Fatalf("start watcher: %v", err)
	}
	defer w.Stop()

	// Start socket server
	server := daemon.NewServer(cfg, database)

	// Handle shutdown
	sigCh := make(chan os.Signal, 1)
	signal.Notify(sigCh, syscall.SIGINT, syscall.SIGTERM)

	go func() {
		<-sigCh
		log.Println("shutting down...")
		server.Stop()
		os.Exit(0)
	}()

	// Start tray (blocks) or just server
	if *noTray {
		log.Println("running without tray")
		if err := server.Start(); err != nil {
			log.Fatalf("server: %v", err)
		}
	} else {
		// Start server in background
		go func() {
			if err := server.Start(); err != nil {
				log.Fatalf("server: %v", err)
			}
		}()

		// Run tray (blocks)
		t := tray.New(cfg, database)
		t.Run(func() {
			server.Stop()
		})
	}
}

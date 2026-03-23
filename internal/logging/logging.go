package logging

import (
	"log/slog"
	"os"
	"strings"
)

var (
	// Logger is the global structured logger
	Logger *slog.Logger

	// SessionID is injected after session initialization
	SessionID string
)

// Init initializes the structured logger based on environment
func Init(verbose bool) {
	level := slog.LevelInfo
	if verbose {
		level = slog.LevelDebug
	}

	// Check environment for log level override
	if envLevel := os.Getenv("KGATE_LOG_LEVEL"); envLevel != "" {
		switch strings.ToLower(envLevel) {
		case "debug":
			level = slog.LevelDebug
		case "info":
			level = slog.LevelInfo
		case "warn", "warning":
			level = slog.LevelWarn
		case "error":
			level = slog.LevelError
		}
	}

	opts := &slog.HandlerOptions{
		Level: level,
	}

	// Use JSON format if KGATE_LOG_JSON is set
	var handler slog.Handler
	if os.Getenv("KGATE_LOG_JSON") == "1" {
		handler = slog.NewJSONHandler(os.Stderr, opts)
	} else {
		handler = slog.NewTextHandler(os.Stderr, opts)
	}

	Logger = slog.New(handler)
	slog.SetDefault(Logger)
}

// SetSessionID updates the session ID for all future log entries
func SetSessionID(id string) {
	SessionID = id
	// Create a new logger with the session ID as a default attribute
	Logger = Logger.With("session_id", id)
	slog.SetDefault(Logger)
}

// WithProject returns a logger with project context
func WithProject(projectID string) *slog.Logger {
	return Logger.With("project_id", projectID)
}

// Debug logs at debug level
func Debug(msg string, args ...any) {
	Logger.Debug(msg, args...)
}

// Info logs at info level
func Info(msg string, args ...any) {
	Logger.Info(msg, args...)
}

// Warn logs at warn level
func Warn(msg string, args ...any) {
	Logger.Warn(msg, args...)
}

// Error logs at error level
func Error(msg string, args ...any) {
	Logger.Error(msg, args...)
}

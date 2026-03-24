package config

import (
	"os"
	"path/filepath"

	"github.com/BurntSushi/toml"
)

type Config struct {
	General       GeneralConfig       `toml:"general"`
	KnowledgeDebt KnowledgeDebtConfig `toml:"knowledge_debt"`
	Voice         VoiceConfig         `toml:"voice"`
	Grading       GradingConfig       `toml:"grading"`
	HTTPBridge    HTTPBridgeConfig    `toml:"http_bridge"`

	// Internal, not serialized
	configDir string `toml:"-"`
}

type GeneralConfig struct {
	ProjectsRoot string `toml:"projects_root"`
	DataDir      string `toml:"data_dir"`
	SocketPath   string `toml:"socket_path"`
}

type KnowledgeDebtConfig struct {
	Threshold           int            `toml:"threshold"`
	DebtPerSprintCleared int           `toml:"debt_per_sprint_cleared"`
	Weights             map[string]int `toml:"weights"`
}

type VoiceConfig struct {
	Enabled           bool   `toml:"enabled"`
	PiperDaemonSocket string `toml:"piper_daemon_socket"`
	MoonshineSocket   string `toml:"moonshine_socket"`
}

type GradingConfig struct {
	PassThreshold     int `toml:"pass_threshold"`
	StreakBonusAt     int `toml:"streak_bonus_at"`
	ShowHintsOnFail   int `toml:"show_hints_on_fail"`
	ShowAnswersOnFail int `toml:"show_answers_on_fail"`
}

type HTTPBridgeConfig struct {
	Enabled bool `toml:"enabled"`
	Port    int  `toml:"port"`
}

func DefaultConfig() *Config {
	home, _ := os.UserHomeDir()

	// XDG Base Directory Specification
	configDir := os.Getenv("XDG_CONFIG_HOME")
	if configDir == "" {
		configDir = filepath.Join(home, ".config")
	}

	dataDir := os.Getenv("XDG_DATA_HOME")
	if dataDir == "" {
		dataDir = filepath.Join(home, ".local", "share")
	}

	runtimeDir := os.Getenv("XDG_RUNTIME_DIR")
	if runtimeDir == "" {
		runtimeDir = "/tmp"
	}

	return &Config{
		General: GeneralConfig{
			ProjectsRoot: filepath.Join(home, "gitZ"),
			DataDir:      filepath.Join(dataDir, "kgate"),
			SocketPath:   filepath.Join(runtimeDir, "kgate.sock"),
		},
		configDir: filepath.Join(configDir, "kgate"),
		KnowledgeDebt: KnowledgeDebtConfig{
			Threshold:           10,
			DebtPerSprintCleared: 3,
			Weights: map[string]int{
				"concept_explained":      1,
				"architecture_decision":  2,
				"bug_fix":                1,
				"new_file":               1,
				"complex_code":           2,
				"why_not_question":       1,
			},
		},
		Voice: VoiceConfig{
			Enabled:           true,
			PiperDaemonSocket: filepath.Join(runtimeDir, "piper-daemon.sock"),
			MoonshineSocket:   "/tmp/moonshine/moonshine.sock",
		},
		Grading: GradingConfig{
			PassThreshold:     60,
			StreakBonusAt:     3,
			ShowHintsOnFail:   1,
			ShowAnswersOnFail: 2,
		},
		HTTPBridge: HTTPBridgeConfig{
			Enabled: false,
			Port:    3001,
		},
	}
}

func Load() (*Config, error) {
	cfg := DefaultConfig()
	configPath := filepath.Join(cfg.configDir, "config.toml")

	// Also check legacy location for migration
	legacyPath := filepath.Join(os.Getenv("HOME"), ".kgate", "config.toml")

	if _, err := os.Stat(configPath); os.IsNotExist(err) {
		// Try legacy location
		if _, err := os.Stat(legacyPath); err == nil {
			if _, err := toml.DecodeFile(legacyPath, cfg); err != nil {
				return nil, err
			}
			return cfg, nil
		}
		return cfg, nil
	}

	if _, err := toml.DecodeFile(configPath, cfg); err != nil {
		return nil, err
	}

	return cfg, nil
}

// ConfigPath returns the path to the config file
func (c *Config) ConfigPath() string {
	return filepath.Join(c.configDir, "config.toml")
}

func (c *Config) Save() error {
	configPath := c.ConfigPath()
	if err := os.MkdirAll(filepath.Dir(configPath), 0750); err != nil {
		return err
	}

	f, err := os.OpenFile(configPath, os.O_WRONLY|os.O_CREATE|os.O_TRUNC, 0640)
	if err != nil {
		return err
	}
	defer f.Close()

	return toml.NewEncoder(f).Encode(c)
}

// DataDir returns the data directory path
func (c *Config) DataDirPath() string {
	return c.General.DataDir
}

func (c *Config) DBPath() string {
	return filepath.Join(c.General.DataDir, "kgate.db")
}

// LegacyDBPath returns the old database location for migration
func (c *Config) LegacyDBPath() string {
	home, _ := os.UserHomeDir()
	return filepath.Join(home, ".kgate", "kgate.db")
}

// AllPaths returns all kgate file paths for backup/restore
func (c *Config) AllPaths() map[string]string {
	return map[string]string{
		"config":   c.ConfigPath(),
		"database": c.DBPath(),
		"data_dir": c.General.DataDir,
	}
}

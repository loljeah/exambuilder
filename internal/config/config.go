package config

import (
	"os"
	"path/filepath"

	"github.com/BurntSushi/toml"
)

type Config struct {
	General      GeneralConfig      `toml:"general"`
	KnowledgeDebt KnowledgeDebtConfig `toml:"knowledge_debt"`
	Voice        VoiceConfig        `toml:"voice"`
	Grading      GradingConfig      `toml:"grading"`
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

func DefaultConfig() *Config {
	home, _ := os.UserHomeDir()
	runtimeDir := os.Getenv("XDG_RUNTIME_DIR")
	if runtimeDir == "" {
		runtimeDir = "/tmp"
	}

	return &Config{
		General: GeneralConfig{
			ProjectsRoot: filepath.Join(home, "gitZ"),
			DataDir:      filepath.Join(home, ".kgate"),
			SocketPath:   filepath.Join(runtimeDir, "kgate.sock"),
		},
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
	}
}

func Load() (*Config, error) {
	cfg := DefaultConfig()
	configPath := filepath.Join(cfg.General.DataDir, "config.toml")

	if _, err := os.Stat(configPath); os.IsNotExist(err) {
		return cfg, nil
	}

	if _, err := toml.DecodeFile(configPath, cfg); err != nil {
		return nil, err
	}

	return cfg, nil
}

func (c *Config) Save() error {
	configPath := filepath.Join(c.General.DataDir, "config.toml")
	if err := os.MkdirAll(filepath.Dir(configPath), 0755); err != nil {
		return err
	}

	f, err := os.Create(configPath)
	if err != nil {
		return err
	}
	defer f.Close()

	return toml.NewEncoder(f).Encode(c)
}

func (c *Config) DBPath() string {
	return filepath.Join(c.General.DataDir, "kgate.db")
}

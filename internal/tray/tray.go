package tray

import (
	"fmt"
	"os"
	"os/exec"
	"path/filepath"

	"fyne.io/systray"
	"github.com/loljeah/exambuilder/assets"
	"github.com/loljeah/exambuilder/internal/config"
	"github.com/loljeah/exambuilder/internal/db"
	"github.com/loljeah/exambuilder/internal/logging"
)

type Tray struct {
	cfg *config.Config
	db  *db.DB

	mOpenGUI *systray.MenuItem
	mStatus  *systray.MenuItem
	mDebt    *systray.MenuItem
	mProfile *systray.MenuItem
	mQuit    *systray.MenuItem

	guiCmd *exec.Cmd
	onQuit func()
}

func New(cfg *config.Config, database *db.DB) *Tray {
	return &Tray{
		cfg: cfg,
		db:  database,
	}
}

func (t *Tray) Run(onQuit func()) {
	t.onQuit = onQuit
	systray.Run(t.onReady, t.onExit)
}

func (t *Tray) onReady() {
	systray.SetTitle("kgate")
	systray.SetTooltip("Knowledge Gate")

	t.mOpenGUI = systray.AddMenuItem("Open GUI", "Open Knowledge Gate window")

	systray.AddSeparator()

	t.mStatus = systray.AddMenuItem("Status: Ready", "Current status")
	t.mStatus.Disable()

	t.mDebt = systray.AddMenuItem("Debt: 0/10", "Knowledge debt")
	t.mDebt.Disable()

	t.mProfile = systray.AddMenuItem("Level 1 • 0 XP", "Profile")
	t.mProfile.Disable()

	systray.AddSeparator()

	t.mQuit = systray.AddMenuItem("Quit", "Stop kgate daemon")

	// Update display
	t.Update("", 0)

	// Handle clicks
	go func() {
		for {
			select {
			case <-t.mOpenGUI.ClickedCh:
				t.launchGUI()
			case <-t.mQuit.ClickedCh:
				t.stopGUI()
				systray.Quit()
				return
			}
		}
	}()
}

func (t *Tray) onExit() {
	t.stopGUI()
	if t.onQuit != nil {
		t.onQuit()
	}
}

// launchGUI starts the GUI application if not already running
func (t *Tray) launchGUI() {
	if t.guiCmd != nil && t.guiCmd.Process != nil {
		// Check if still running
		if t.guiCmd.ProcessState == nil {
			logging.Info("GUI already running")
			return
		}
	}

	// Find GUI binary - check multiple locations
	guiPaths := []string{
		// Relative to daemon binary
		"./kgate-gui",
		"./gui/build/bin/kgate-gui",
		// System install paths
		"/usr/local/bin/kgate-gui",
		"/usr/bin/kgate-gui",
	}

	// Add path relative to executable
	if exe, err := os.Executable(); err == nil {
		dir := filepath.Dir(exe)
		guiPaths = append([]string{
			filepath.Join(dir, "kgate-gui"),
			filepath.Join(dir, "..", "gui", "build", "bin", "kgate-gui"),
		}, guiPaths...)
	}

	var guiPath string
	for _, p := range guiPaths {
		if _, err := os.Stat(p); err == nil {
			guiPath = p
			break
		}
	}

	if guiPath == "" {
		logging.Error("GUI binary not found", "searched", guiPaths)
		return
	}

	logging.Info("launching GUI", "path", guiPath)
	t.guiCmd = exec.Command(guiPath)
	t.guiCmd.Stdout = os.Stdout
	t.guiCmd.Stderr = os.Stderr

	if err := t.guiCmd.Start(); err != nil {
		logging.Error("failed to launch GUI", "error", err)
		t.guiCmd = nil
		return
	}

	// Wait for process in background to clean up when it exits
	go func() {
		if t.guiCmd != nil {
			t.guiCmd.Wait()
			logging.Info("GUI exited")
		}
	}()
}

// stopGUI terminates the GUI if running
func (t *Tray) stopGUI() {
	if t.guiCmd != nil && t.guiCmd.Process != nil {
		logging.Info("stopping GUI")
		t.guiCmd.Process.Signal(os.Interrupt)
		t.guiCmd = nil
	}
}

// Update refreshes the tray display
func (t *Tray) Update(projectID string, debt int) {
	threshold := t.cfg.KnowledgeDebt.Threshold

	// Update icon based on debt level
	percent := 0
	if threshold > 0 {
		percent = (debt * 100) / threshold
	}

	switch {
	case percent >= 80:
		systray.SetIcon(assets.IconRed)
		systray.SetTooltip("High debt - take an exam!")
	case percent >= 50:
		systray.SetIcon(assets.IconYellow)
		systray.SetTooltip("Knowledge Gate - debt building")
	default:
		systray.SetIcon(assets.IconGreen)
		systray.SetTooltip("Knowledge Gate - ready")
	}

	// Update menu items
	t.mDebt.SetTitle(fmt.Sprintf("Debt: %d/%d", debt, threshold))

	if profile, err := t.db.GetProfile(); err == nil {
		t.mProfile.SetTitle(fmt.Sprintf("Level %d • %d XP • 🔥%d",
			profile.Level, profile.TotalXP, profile.CurrentStreak))
	}

	if projectID != "" {
		if p, err := t.db.GetProject(projectID); err == nil {
			t.mStatus.SetTitle(fmt.Sprintf("Project: %s", p.Name))
		}
	} else {
		t.mStatus.SetTitle("No active project")
	}
}

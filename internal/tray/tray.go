package tray

import (
	"fmt"

	"fyne.io/systray"
	"github.com/loljeah/exambuilder/internal/config"
	"github.com/loljeah/exambuilder/internal/db"
)

// Icons (base64 encoded PNGs would go here, using placeholders)
var (
	iconGreen  = []byte{} // debt < 50%
	iconYellow = []byte{} // debt 50-80%
	iconRed    = []byte{} // debt >= 80%
)

type Tray struct {
	cfg *config.Config
	db  *db.DB

	mStatus  *systray.MenuItem
	mDebt    *systray.MenuItem
	mProfile *systray.MenuItem
	mQuit    *systray.MenuItem

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

	t.mStatus = systray.AddMenuItem("Status: Ready", "Current status")
	t.mStatus.Disable()

	systray.AddSeparator()

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
		for range t.mQuit.ClickedCh {
			systray.Quit()
		}
	}()
}

func (t *Tray) onExit() {
	if t.onQuit != nil {
		t.onQuit()
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
		systray.SetIcon(iconRed)
		systray.SetTooltip("⚠️ High debt - take an exam!")
	case percent >= 50:
		systray.SetIcon(iconYellow)
		systray.SetTooltip("Knowledge Gate - debt building")
	default:
		systray.SetIcon(iconGreen)
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

package main

import (
	"context"
	"embed"
	"flag"

	"github.com/loljeah/exambuilder/internal/config"
	"github.com/wailsapp/wails/v2"
	"github.com/wailsapp/wails/v2/pkg/options"
	"github.com/wailsapp/wails/v2/pkg/options/assetserver"
	"github.com/wailsapp/wails/v2/pkg/options/linux"
)

//go:embed all:frontend/dist
var assets embed.FS

func main() {
	// Parse command-line flags
	httpPort := flag.Int("http", 0, "HTTP bridge port (overrides config, 0=use config)")
	flag.Parse()

	// Load config early for HTTP bridge decision
	cfg, _ := config.Load()

	// Create application with options
	app := NewApp()

	// Start HTTP bridge if enabled (flag overrides config)
	var bridge *HTTPBridge
	if *httpPort > 0 {
		// Flag explicitly set - use flag value
		bridge = NewHTTPBridge(app, *httpPort)
		bridge.Start()
	} else if cfg != nil && cfg.HTTPBridge.Enabled && cfg.HTTPBridge.Port > 0 {
		// Use config value
		bridge = NewHTTPBridge(app, cfg.HTTPBridge.Port)
		bridge.Start()
	}

	err := wails.Run(&options.App{
		Title:     "Knowledge Gate",
		Width:     1200,
		Height:    800,
		MinWidth:  900,
		MinHeight: 600,
		Frameless: false, // Use native window frame (enables normal clicking)
		AssetServer: &assetserver.Options{
			Assets: assets,
		},
		BackgroundColour: &options.RGBA{R: 30, G: 25, B: 40, A: 1},
		OnStartup:        app.startup,
		OnShutdown: func(ctx context.Context) {
			if bridge != nil {
				bridge.Stop(ctx)
			}
			app.shutdown(ctx)
		},
		Bind: []interface{}{
			app,
		},
		Linux: &linux.Options{
			ProgramName:         "Knowledge Gate",
			WebviewGpuPolicy:    linux.WebviewGpuPolicyOnDemand,
			WindowIsTranslucent: false,
		},
	})

	if err != nil {
		println("Error:", err.Error())
	}
}

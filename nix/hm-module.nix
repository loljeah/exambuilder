{ config, lib, pkgs, ... }:

let
  cfg = config.services.kgate;
  tomlFormat = pkgs.formats.toml { };

  # Build the full config TOML from settings + extraConfig
  configFile = tomlFormat.generate "kgate-config.toml" (
    lib.recursiveUpdate cfg.settings cfg.extraConfig
  );

  # Wrapper script: ensure daemon running, then launch GUI
  guiLauncherScript = pkgs.writeShellScript "kgate-gui-launcher" ''
    ${pkgs.systemd}/bin/systemctl --user is-active kgate-daemon.service &>/dev/null \
      || ${pkgs.systemd}/bin/systemctl --user start kgate-daemon.service
    sleep 0.5
    exec ${cfg.guiPackage}/bin/kgate-gui "$@"
  '';

  # Toggle script: start/stop daemon with notification
  daemonToggleScript = pkgs.writeShellScript "kgate-daemon-toggle" ''
    if ${pkgs.systemd}/bin/systemctl --user is-active kgate-daemon.service &>/dev/null; then
      ${pkgs.systemd}/bin/systemctl --user stop kgate-daemon.service
      ${pkgs.libnotify}/bin/notify-send -a "Knowledge Gate" "Daemon stopped" -t 2000
    else
      ${pkgs.systemd}/bin/systemctl --user start kgate-daemon.service
      ${pkgs.libnotify}/bin/notify-send -a "Knowledge Gate" "Daemon started" -t 2000
    fi
  '';

in {
  options.services.kgate = {
    enable = lib.mkEnableOption "Knowledge Gate exam system";

    package = lib.mkOption {
      type = lib.types.package;
      description = "Package providing kgate-daemon and kgatectl.";
    };

    guiPackage = lib.mkOption {
      type = lib.types.nullOr lib.types.package;
      default = null;
      description = "Package providing kgate-gui. Set to null to skip GUI desktop entries.";
    };

    settings = lib.mkOption {
      type = lib.types.submodule {
        freeformType = tomlFormat.type;

        options = {
          general = {
            projects_root = lib.mkOption {
              type = lib.types.str;
              default = "";
              description = "Root directory for projects. Empty = Go default (~/gitZ).";
            };
            data_dir = lib.mkOption {
              type = lib.types.str;
              default = "";
              description = "Data directory. Empty = XDG_DATA_HOME/kgate.";
            };
            socket_path = lib.mkOption {
              type = lib.types.str;
              default = "";
              description = "Unix socket path. Empty = XDG_RUNTIME_DIR/kgate.sock.";
            };
          };

          knowledge_debt = {
            threshold = lib.mkOption {
              type = lib.types.int;
              default = 10;
              description = "Debt threshold before read-only mode.";
            };
            debt_per_sprint_cleared = lib.mkOption {
              type = lib.types.int;
              default = 3;
              description = "Debt reduced per cleared sprint.";
            };
            weights = lib.mkOption {
              type = lib.types.attrsOf lib.types.int;
              default = {
                concept_explained = 1;
                architecture_decision = 2;
                bug_fix = 1;
                new_file = 1;
                complex_code = 2;
                why_not_question = 1;
              };
              description = "Debt weights per action type.";
            };
          };

          voice = {
            enabled = lib.mkOption {
              type = lib.types.bool;
              default = true;
              description = "Enable voice mode.";
            };
            piper_daemon_socket = lib.mkOption {
              type = lib.types.str;
              default = "";
              description = "Piper TTS daemon socket. Empty = XDG_RUNTIME_DIR/piper-daemon.sock.";
            };
            moonshine_socket = lib.mkOption {
              type = lib.types.str;
              default = "/tmp/moonshine/moonshine.sock";
              description = "Moonshine STT socket path.";
            };
          };

          grading = {
            pass_threshold = lib.mkOption {
              type = lib.types.int;
              default = 60;
              description = "Minimum percentage to pass a sprint.";
            };
            streak_bonus_at = lib.mkOption {
              type = lib.types.int;
              default = 3;
              description = "Streak length for bonus coins.";
            };
            show_hints_on_fail = lib.mkOption {
              type = lib.types.int;
              default = 1;
              description = "Show hints after N failed attempts.";
            };
            show_answers_on_fail = lib.mkOption {
              type = lib.types.int;
              default = 2;
              description = "Show answers after N failed attempts.";
            };
          };

          http_bridge = {
            enabled = lib.mkOption {
              type = lib.types.bool;
              default = false;
              description = "Enable HTTP bridge for external integrations.";
            };
            port = lib.mkOption {
              type = lib.types.port;
              default = 3001;
              description = "HTTP bridge port.";
            };
          };

          ollama = {
            base_url = lib.mkOption {
              type = lib.types.str;
              default = "http://localhost:11434";
              description = "Ollama API base URL.";
            };
            model = lib.mkOption {
              type = lib.types.str;
              default = "llama3.1:8b";
              description = "Default LLM model for exam generation.";
            };
            timeout_seconds = lib.mkOption {
              type = lib.types.int;
              default = 120;
              description = "Request timeout in seconds.";
            };
            max_retries = lib.mkOption {
              type = lib.types.int;
              default = 2;
              description = "Maximum retry attempts on failure.";
            };
          };
        };
      };
      default = { };
      description = "kgate configuration. Maps directly to config.toml sections.";
    };

    extraConfig = lib.mkOption {
      type = tomlFormat.type;
      default = { };
      description = "Extra configuration merged into config.toml. Escape hatch for options not yet covered.";
    };

    desktopEntries = lib.mkOption {
      type = lib.types.bool;
      default = true;
      description = "Generate .desktop entries for rofi/launcher discovery.";
    };
  };

  config = lib.mkIf cfg.enable {
    # Install daemon + ctl, and optionally the GUI
    home.packages = [ cfg.package ]
      ++ lib.optional (cfg.guiPackage != null) cfg.guiPackage;

    # Generate config.toml
    xdg.configFile."kgate/config.toml".source = configFile;

    # Systemd user service for the daemon
    systemd.user.services.kgate-daemon = {
      Unit = {
        Description = "Knowledge Gate Daemon";
        After = [ "graphical-session.target" ];
        PartOf = [ "graphical-session.target" ];
      };
      Service = {
        Type = "simple";
        ExecStart = "${cfg.package}/bin/kgate-daemon";
        Restart = "on-failure";
        RestartSec = 5;

        # Hardening
        NoNewPrivileges = true;
        ProtectSystem = "strict";
        ProtectHome = "read-only";
        ReadWritePaths = [
          "%h/.local/share/kgate"
          "%h/.config/kgate"
          "%t"
        ];
        PrivateTmp = true;
        ProtectKernelTunables = true;
        ProtectKernelModules = true;
        ProtectControlGroups = true;
      };
      Install = {
        WantedBy = [ "graphical-session.target" ];
      };
    };

    # Desktop entries for rofi discovery
    xdg.desktopEntries = lib.mkIf cfg.desktopEntries (
      lib.optionalAttrs (cfg.guiPackage != null) {
        kgate = {
          name = "Knowledge Gate";
          comment = "Gamified exam and knowledge tracking";
          exec = "${guiLauncherScript}";
          icon = "accessories-text-editor";
          type = "Application";
          categories = [ "Education" "Development" ];
        };
      } // {
        kgate-daemon-toggle = {
          name = "kgate daemon toggle";
          comment = "Start or stop the Knowledge Gate daemon";
          exec = "${daemonToggleScript}";
          icon = "system-run";
          type = "Application";
          categories = [ "System" ];
        };
      }
    );
  };
}

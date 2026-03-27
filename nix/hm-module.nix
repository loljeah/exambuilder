{ config, lib, pkgs, ... }:

let
  cfg = config.services.kgate;
  tomlFormat = pkgs.formats.toml { };

  # Build the full config TOML from settings + extraConfig
  configFile = tomlFormat.generate "kgate-config.toml" (
    lib.recursiveUpdate cfg.settings cfg.extraConfig
  );

in {
  options.services.kgate = {
    enable = lib.mkEnableOption "Knowledge Gate exam system";

    package = lib.mkOption {
      type = lib.types.package;
      description = "Package providing kgate.";
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

    desktopEntry = lib.mkOption {
      type = lib.types.bool;
      default = true;
      description = "Generate .desktop entry for rofi/launcher discovery.";
    };
  };

  config = lib.mkIf cfg.enable {
    # Install package
    home.packages = [ cfg.package ];

    # Generate config.toml
    xdg.configFile."kgate/config.toml".source = configFile;

    # Desktop entry for rofi discovery
    xdg.desktopEntries = lib.mkIf cfg.desktopEntry {
      kgate = {
        name = "Knowledge Gate";
        comment = "Gamified exam and knowledge tracking";
        exec = "${cfg.package}/bin/kgate";
        icon = "accessories-text-editor";
        type = "Application";
        categories = [ "Education" "Development" ];
      };
    };
  };
}

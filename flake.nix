{
  description = "kgate - Knowledge Gate exam system";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    let
      # System-independent outputs
      systemIndependent = {
        homeManagerModules.default = ./nix/hm-module.nix;

        overlays.default = final: prev: {
          kgate = self.packages.${prev.system}.default;
          kgate-cli = self.packages.${prev.system}.daemon;
          kgate-gui = self.packages.${prev.system}.gui;
        };
      };

      # Per-system outputs
      perSystem = flake-utils.lib.eachDefaultSystem (system:
        let
          pkgs = nixpkgs.legacyPackages.${system};

          version = "0.3.0";
          vendorHash = "sha256-fjKOOluDPxtghcfPHDsHfoNhtRuc7nQChZBnYPjRbBU=";

          # ── Stage 1: Build Svelte frontend ──
          kgate-frontend = pkgs.buildNpmPackage {
            pname = "kgate-frontend";
            inherit version;
            src = ./gui/frontend;
            npmDepsHash = "sha256-M56gPlXGsiD71U3ru1r//vKHGodTEKKZzvch4pmzX5k=";
            buildPhase = ''
              npx vite build
            '';
            installPhase = ''
              mkdir -p $out
              cp -r dist/* $out/
            '';
          };

          # ── Stage 2: Build GUI binary (Go + embedded frontend) ──
          kgate-gui = pkgs.buildGoModule {
            pname = "kgate-gui";
            inherit version vendorHash;
            src = ./.;
            subPackages = [ "gui" ];
            tags = [ "webkit2_41" ];

            nativeBuildInputs = with pkgs; [ pkg-config ];
            buildInputs = with pkgs; [
              webkitgtk_4_1
              libsoup_3
              gtk3
              glib
            ];

            # Inject pre-built frontend before Go embed runs
            preBuild = ''
              mkdir -p gui/frontend/dist
              cp -r ${kgate-frontend}/* gui/frontend/dist/
            '';

            # buildGoModule names the binary after the subPackage dir
            postInstall = ''
              mv $out/bin/gui $out/bin/kgate-gui
            '';

            meta = with pkgs.lib; {
              description = "Knowledge Gate - GUI (Wails)";
              license = licenses.mit;
            };
          };

          # ── Stage 3: Build daemon + ctl ──
          kgate-cli = pkgs.buildGoModule {
            pname = "kgate";
            inherit version vendorHash;
            src = ./.;
            subPackages = [ "cmd/kgate-daemon" "cmd/kgatectl" ];

            nativeBuildInputs = with pkgs; [ pkg-config ];
            buildInputs = with pkgs; [
              libayatana-appindicator
              gtk3
              glib
            ];

            meta = with pkgs.lib; {
              description = "Knowledge Gate - daemon and CLI";
              license = licenses.mit;
            };
          };

        in {
          # Combined package: all binaries
          packages.default = pkgs.symlinkJoin {
            name = "kgate-${version}";
            paths = [ kgate-cli kgate-gui ];
            meta.description = "Knowledge Gate - complete package";
          };

          packages.daemon = kgate-cli;
          packages.gui = kgate-gui;
          packages.frontend = kgate-frontend;

          devShells.default = pkgs.mkShell {
            buildInputs = with pkgs; [
              # Go
              go
              gopls
              gotools
              go-tools
              sqlite
              pkg-config
              libayatana-appindicator
              gtk3
              glib

              # Wails GUI
              wails
              nodejs_20
              nodePackages.npm
              webkitgtk_4_1
              libsoup_3

              # Pixel art
              python3
              python3Packages.pillow

              # LLM generation
              ollama
            ];

            GOFLAGS = "-tags=webkit2_41";

            shellHook = ''
              echo "kgate dev environment (Go + Wails)"
              echo "  go build ./cmd/kgate-daemon"
              echo "  go build ./cmd/kgatectl"
              echo "  cd gui && wails dev"
              echo ""
              if command -v ollama &>/dev/null; then
                if curl -sf http://localhost:11434/api/tags &>/dev/null 2>&1; then
                  _mc=$(curl -s http://localhost:11434/api/tags 2>/dev/null | grep -c '"name"' || echo 0)
                  echo "  Ollama running ($_mc models)"
                else
                  echo "  Ollama installed but not running — start: ollama serve"
                fi
              else
                echo "  Ollama not found"
              fi
              echo "  scripts/setup-ollama.sh for guided LLM setup"
            '';
          };
        }
      );

    in
      # Merge system-independent and per-system outputs
      systemIndependent // perSystem;
}

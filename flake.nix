{
  description = "kgate - Knowledge Gate exam system";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
      in
      {
        packages.default = pkgs.buildGoModule {
          pname = "kgate";
          version = "0.2.0";
          src = ./.;
          vendorHash = null; # Update after go mod tidy

          nativeBuildInputs = with pkgs; [
            pkg-config
          ];

          buildInputs = with pkgs; [
            libayatana-appindicator
            gtk3
            glib
          ];

          subPackages = [ "cmd/kgate-daemon" "cmd/kgatectl" ];

          meta = with pkgs.lib; {
            description = "Knowledge Gate - gamified exam system";
            license = licenses.mit;
            maintainers = [ ];
          };
        };

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

            # Pixel art
            python3
            python3Packages.pillow
          ];

          shellHook = ''
            echo "kgate dev environment (Go + Wails)"
            echo "  go build ./cmd/kgate-daemon"
            echo "  go build ./cmd/kgatectl"
            echo "  cd gui && wails dev"
          '';
        };
      }
    );
}

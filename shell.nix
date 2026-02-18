{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell {
  name = "kgate-dev";

  buildInputs = with pkgs; [
    # Rust toolchain
    rustc
    cargo
    rust-analyzer
    clippy          # Linter
    rustfmt         # Formatter

    # Cargo extensions
    cargo-watch     # Auto-rebuild on save
    cargo-edit      # `cargo add/rm` commands
    sqlx-cli        # DB migrations

    # System deps (Rust crates link against these)
    sqlite
    openssl
    pkg-config

    # Dev tools
    watchexec       # File watcher
    just            # Task runner (like make)
  ];

  # Help Rust find system libs
  SQLITE3_DIR = "${pkgs.sqlite.dev}";
  OPENSSL_DIR = "${pkgs.openssl.dev}";
  PKG_CONFIG_PATH = "${pkgs.sqlite.dev}/lib/pkgconfig:${pkgs.openssl.dev}/lib/pkgconfig";

  shellHook = ''
    echo ""
    echo "🦀 kgate dev environment"
    echo "   cargo build    — compile"
    echo "   cargo run      — run CLI"
    echo "   cargo watch -x run — auto-rebuild"
    echo "   sqlx migrate run — run DB migrations"
    echo ""
  '';
}

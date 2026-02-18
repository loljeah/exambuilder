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

    # Voice/Audio deps
    alsa-lib        # Linux audio
    libpulseaudio   # PulseAudio
    espeak-ng       # TTS fallback
    piper-tts       # Neural TTS (if available)
    vosk            # Offline STT

    # Dev tools
    watchexec       # File watcher
    just            # Task runner (like make)
  ];

  # Help Rust find system libs
  SQLITE3_DIR = "${pkgs.sqlite.dev}";
  OPENSSL_DIR = "${pkgs.openssl.dev}";
  PKG_CONFIG_PATH = "${pkgs.sqlite.dev}/lib/pkgconfig:${pkgs.openssl.dev}/lib/pkgconfig:${pkgs.alsa-lib.dev}/lib/pkgconfig";

  # Audio libs
  ALSA_PLUGIN_DIR = "${pkgs.alsa-plugins}/lib/alsa-lib";

  shellHook = ''
    echo ""
    echo "🦀 kgate dev environment"
    echo "   just build     — compile"
    echo "   just run       — run CLI"
    echo "   just voice     — test voice mode"
    echo "   just dev       — auto-rebuild"
    echo ""
  '';
}

{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell {
  name = "kgate-dev";

  buildInputs = with pkgs; [
    # Rust toolchain
    rustc
    cargo
    rust-analyzer
    clippy
    rustfmt

    # Cargo extensions
    cargo-watch
    cargo-edit

    # System deps (Rust crates link against these)
    sqlite
    openssl
    pkg-config

    # Audio deps (for voice mode)
    alsa-lib
    libpulseaudio
    alsa-utils      # arecord/aplay for recording
    espeak-ng       # Fallback TTS
    piper-tts       # Neural TTS
    whisper-cpp     # STT (whisper.cpp)

    # Dev tools
    watchexec
    just
  ];

  # Help Rust find system libs
  SQLITE3_DIR = "${pkgs.sqlite.dev}";
  OPENSSL_DIR = "${pkgs.openssl.dev}";
  PKG_CONFIG_PATH = "${pkgs.sqlite.dev}/lib/pkgconfig:${pkgs.openssl.dev}/lib/pkgconfig:${pkgs.alsa-lib.dev}/lib/pkgconfig";

  shellHook = ''
    echo ""
    echo "🦀 kgate dev environment"
    echo "   just build     — compile"
    echo "   just run       — run CLI"
    echo "   just dev       — auto-rebuild"
    echo ""
  '';
}

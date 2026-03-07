//! Voice mode for hands-free exam taking
//!
//! Provides TTS (text-to-speech) for reading questions and STT (speech-to-text)
//! for accepting spoken answers. Enabled via `--voice`/`-v` flag on take command.

pub mod config;
pub mod setup;
pub mod stt;
pub mod tts;

pub use setup::run_setup_wizard;

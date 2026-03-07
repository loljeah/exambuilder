//! Speech-to-Text implementations
//!
//! Supports whisper.cpp for transcription

use anyhow::{Context, Result};
use std::process::{Command, Stdio};

use super::config::{CalibrationConfig, SttConfig};

/// Speech-to-text trait
pub trait SpeechToText {
    /// Listen for speech and return transcription
    fn listen(&self) -> Result<String>;

    /// Listen with timeout, returns None on timeout
    fn listen_with_timeout(&self, timeout_ms: u32) -> Result<Option<String>>;
}

/// Whisper.cpp STT implementation
pub struct WhisperStt {
    model: String,
    language: String,
    sample_rate: u32,
    max_wait_ms: u32,
    mic_device: Option<String>,
}

impl WhisperStt {
    pub fn new(stt_config: &SttConfig, calib_config: &CalibrationConfig) -> Self {
        Self {
            model: stt_config.model.clone(),
            language: stt_config.language.clone(),
            sample_rate: calib_config.sample_rate,
            max_wait_ms: calib_config.max_wait_time_ms,
            mic_device: calib_config.microphone_device.clone(),
        }
    }

    pub fn from_defaults() -> Self {
        Self {
            model: "base.en".to_string(),
            language: "en".to_string(),
            sample_rate: 16000,
            max_wait_ms: 5000,
            mic_device: None,
        }
    }

    /// Record audio from microphone
    fn record_audio(&self, duration_ms: u32) -> Result<Vec<u8>> {
        let duration_secs = (duration_ms as f32 / 1000.0).ceil() as u32;

        let mut cmd = Command::new("arecord");
        cmd.args([
            "-f",
            "S16_LE",
            "-r",
            &self.sample_rate.to_string(),
            "-c",
            "1",
            "-t",
            "wav",
            "-d",
            &duration_secs.to_string(),
            "-q", // quiet
            "-",  // output to stdout
        ]);

        if let Some(ref device) = self.mic_device {
            cmd.args(["-D", device]);
        }

        let output = cmd.stdout(Stdio::piped()).stderr(Stdio::null()).output()?;

        Ok(output.stdout)
    }

    /// Transcribe audio using whisper
    fn transcribe(&self, audio_data: &[u8]) -> Result<String> {
        // Write audio to temp file
        let temp_path = std::env::temp_dir().join("kgate_voice_input.wav");
        std::fs::write(&temp_path, audio_data)?;

        // Try whisper-cpp first, then fall back to whisper
        let whisper_cmd = if which::which("whisper-cpp").is_ok() {
            "whisper-cpp"
        } else if which::which("whisper").is_ok() {
            "whisper"
        } else {
            return Err(anyhow::anyhow!("No whisper binary found"));
        };

        let output = Command::new(whisper_cmd)
            .args([
                "-m",
                &self.model,
                "-l",
                &self.language,
                "-f",
                temp_path.to_str().unwrap(),
                "--no-timestamps",
                "-otxt",
            ])
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .output()
            .context("Failed to run whisper")?;

        // Clean up temp file
        let _ = std::fs::remove_file(&temp_path);

        // Parse output
        let text = String::from_utf8_lossy(&output.stdout)
            .trim()
            .to_string();

        Ok(text)
    }
}

impl SpeechToText for WhisperStt {
    fn listen(&self) -> Result<String> {
        // Record for default timeout
        let audio = self.record_audio(self.max_wait_ms)?;
        self.transcribe(&audio)
    }

    fn listen_with_timeout(&self, timeout_ms: u32) -> Result<Option<String>> {
        let audio = self.record_audio(timeout_ms)?;
        if audio.is_empty() {
            return Ok(None);
        }
        let text = self.transcribe(&audio)?;
        if text.is_empty() {
            Ok(None)
        } else {
            Ok(Some(text))
        }
    }
}

/// Parse spoken answer to option letter (A, B, C, D)
pub fn parse_voice_answer(text: &str) -> Option<char> {
    let text_lower = text.to_lowercase().trim().to_string();

    // Direct letter match
    if text_lower == "a" || text_lower.starts_with("a ") || text_lower.contains("option a") {
        return Some('A');
    }
    if text_lower == "b" || text_lower.starts_with("b ") || text_lower.contains("option b") {
        return Some('B');
    }
    if text_lower == "c" || text_lower.starts_with("c ") || text_lower.contains("option c") {
        return Some('C');
    }
    if text_lower == "d" || text_lower.starts_with("d ") || text_lower.contains("option d") {
        return Some('D');
    }

    // Word matching
    if text_lower.contains("first") || text_lower.contains("one") {
        return Some('A');
    }
    if text_lower.contains("second") || text_lower.contains("two") {
        return Some('B');
    }
    if text_lower.contains("third") || text_lower.contains("three") {
        return Some('C');
    }
    if text_lower.contains("fourth") || text_lower.contains("four") || text_lower.contains("last")
    {
        return Some('D');
    }

    None
}

/// Create an STT instance based on config
pub fn create_stt(
    stt_config: &SttConfig,
    calib_config: &CalibrationConfig,
) -> Box<dyn SpeechToText + Send> {
    Box::new(WhisperStt::new(stt_config, calib_config))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_voice_answer() {
        assert_eq!(parse_voice_answer("A"), Some('A'));
        assert_eq!(parse_voice_answer("option b"), Some('B'));
        assert_eq!(parse_voice_answer("the first one"), Some('A'));
        assert_eq!(parse_voice_answer("three"), Some('C'));
        assert_eq!(parse_voice_answer("gibberish"), None);
    }
}

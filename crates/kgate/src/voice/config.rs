//! Voice configuration handling
//!
//! Loads and saves voice settings from ~/.kgate/voice.toml

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Voice configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceConfig {
    pub general: GeneralConfig,
    pub tts: TtsConfig,
    pub stt: SttConfig,
    pub calibration: CalibrationConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralConfig {
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TtsConfig {
    pub engine: TtsEngine,
    pub voice: String,
    pub speed: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub piper_model: Option<String>,
    /// Kokoro voice name (e.g., "af_bella")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kokoro_voice: Option<String>,
    /// Piper noise scale — generator noise, lower = calmer (default 0.4)
    #[serde(default = "default_noise_scale", skip_serializing_if = "Option::is_none")]
    pub piper_noise_scale: Option<f32>,
    /// Piper noise_w scale — pitch variation, lower = fewer pitch spikes (default 0.2)
    #[serde(default = "default_noise_w_scale", skip_serializing_if = "Option::is_none")]
    pub piper_noise_w_scale: Option<f32>,
    /// Piper length scale — speaking rate, higher = slower (default 1.05)
    #[serde(default = "default_length_scale", skip_serializing_if = "Option::is_none")]
    pub piper_length_scale: Option<f32>,
}

fn default_noise_scale() -> Option<f32> { Some(0.4) }
fn default_noise_w_scale() -> Option<f32> { Some(0.2) }
fn default_length_scale() -> Option<f32> { Some(1.05) }

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum TtsEngine {
    EspeakNg,
    Piper,
    Kokoro,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SttConfig {
    pub engine: SttEngine,
    pub model: String,
    pub language: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum SttEngine {
    Whisper,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalibrationConfig {
    pub silence_threshold: f32,
    pub max_wait_time_ms: u32,
    pub confirm_answers: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub microphone_device: Option<String>,
    pub sample_rate: u32,
}

impl Default for VoiceConfig {
    fn default() -> Self {
        Self {
            general: GeneralConfig { enabled: true },
            tts: TtsConfig {
                engine: TtsEngine::Piper,
                voice: "en-gb".to_string(),
                speed: 150,
                piper_model: Some(
                    dirs::home_dir()
                        .expect("No home directory")
                        .join(".kgate/voices/piper/en_GB-cori-high.onnx")
                        .to_string_lossy()
                        .to_string(),
                ),
                kokoro_voice: None,
                piper_noise_scale: Some(0.4),
                piper_noise_w_scale: Some(0.2),
                piper_length_scale: Some(1.05),
            },
            stt: SttConfig {
                engine: SttEngine::Whisper,
                model: "base.en".to_string(),
                language: "en".to_string(),
            },
            calibration: CalibrationConfig {
                silence_threshold: 0.01,
                max_wait_time_ms: 5000,
                confirm_answers: true,
                microphone_device: None,
                sample_rate: 16000,
            },
        }
    }
}

impl VoiceConfig {
    /// Get path to voice config file
    pub fn config_path() -> PathBuf {
        dirs::home_dir()
            .expect("No home directory")
            .join(".kgate")
            .join("voice.toml")
    }

    /// Load config from file, or return default if not found
    pub fn load() -> Result<Self> {
        let path = Self::config_path();
        if path.exists() {
            let content = std::fs::read_to_string(&path)?;
            let config: VoiceConfig = toml::from_str(&content)?;
            Ok(config)
        } else {
            Ok(Self::default())
        }
    }

    /// Save config to file
    pub fn save(&self) -> Result<()> {
        let path = Self::config_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = toml::to_string_pretty(self)?;
        std::fs::write(&path, content)?;
        Ok(())
    }

    /// Check if voice mode is configured and enabled
    pub fn is_configured(&self) -> bool {
        self.general.enabled && Self::config_path().exists()
    }

    /// Detect available TTS engines on the system
    pub fn detect_tts_engines() -> Vec<TtsEngine> {
        let mut engines = Vec::new();

        // Check for kokoro (best quality)
        if which::which("kokoro-tts").is_ok() || Self::has_kokoro_python() {
            engines.push(TtsEngine::Kokoro);
        }

        // Check for piper
        if which::which("piper").is_ok() {
            engines.push(TtsEngine::Piper);
        }

        // Check for espeak-ng (fallback)
        if which::which("espeak-ng").is_ok() {
            engines.push(TtsEngine::EspeakNg);
        }

        engines
    }

    /// Check if kokoro Python package is available
    pub fn has_kokoro_python() -> bool {
        std::process::Command::new("python3")
            .args(["-c", "import kokoro"])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
    }

    /// Detect available STT engines on the system
    pub fn detect_stt_engines() -> Vec<SttEngine> {
        let mut engines = Vec::new();

        // Check for whisper
        if which::which("whisper-cpp").is_ok() || which::which("whisper").is_ok() {
            engines.push(SttEngine::Whisper);
        }

        engines
    }

    /// Check if arecord is available for microphone recording
    pub fn has_arecord() -> bool {
        which::which("arecord").is_ok()
    }

    /// Check if aplay is available for audio playback
    #[allow(dead_code)]
    pub fn has_aplay() -> bool {
        which::which("aplay").is_ok()
    }
}

impl std::fmt::Display for TtsEngine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TtsEngine::EspeakNg => write!(f, "espeak-ng"),
            TtsEngine::Piper => write!(f, "piper"),
            TtsEngine::Kokoro => write!(f, "kokoro"),
        }
    }
}

impl std::fmt::Display for SttEngine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SttEngine::Whisper => write!(f, "whisper"),
        }
    }
}

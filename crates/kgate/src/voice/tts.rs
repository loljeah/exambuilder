//! Text-to-Speech implementations
//!
//! Supports kokoro (best quality), piper (neural), and espeak-ng (fallback)

use anyhow::Result;
use console::style;
use std::process::{Command, Stdio};

use super::config::{TtsConfig, TtsEngine, VoiceConfig};

/// Text-to-speech trait
pub trait TextToSpeech {
    /// Speak text (non-blocking, spawns background process)
    fn speak(&self, text: &str) -> Result<()>;

    /// Speak text and wait for completion
    fn speak_blocking(&self, text: &str) -> Result<()>;
}

/// espeak-ng TTS implementation
pub struct EspeakTts {
    voice: String,
    speed: u32,
}

impl EspeakTts {
    pub fn new(config: &TtsConfig) -> Self {
        Self {
            voice: config.voice.clone(),
            speed: config.speed,
        }
    }

    #[allow(dead_code)]
    pub fn from_defaults() -> Self {
        Self {
            voice: "en-us".to_string(),
            speed: 150,
        }
    }

    pub fn is_available() -> bool {
        which::which("espeak-ng").is_ok()
    }
}

impl TextToSpeech for EspeakTts {
    fn speak(&self, text: &str) -> Result<()> {
        let cleaned = format_text_for_speech(text);
        let speed = self.speed.to_string();
        let voice = self.voice.clone();

        std::thread::spawn(move || {
            let _ = Command::new("espeak-ng")
                .args(["-v", &voice, "-s", &speed, &cleaned])
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn();
        });

        Ok(())
    }

    fn speak_blocking(&self, text: &str) -> Result<()> {
        let cleaned = format_text_for_speech(text);
        let status = Command::new("espeak-ng")
            .args(["-v", &self.voice, "-s", &self.speed.to_string(), &cleaned])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()?;
        if !status.success() {
            anyhow::bail!("espeak-ng exited with {}", status);
        }
        Ok(())
    }
}

/// Default directory for piper voice models
fn piper_voices_dir() -> std::path::PathBuf {
    dirs::home_dir()
        .expect("No home directory")
        .join(".kgate")
        .join("voices")
        .join("piper")
}

/// Find a piper model: use configured path, or auto-detect from ~/.kgate/voices/piper/
fn resolve_piper_model(configured: &Option<String>) -> Option<String> {
    // Use explicitly configured model if set
    if let Some(ref m) = configured {
        if !m.is_empty() && std::path::Path::new(m).exists() {
            return Some(m.clone());
        }
    }

    // Auto-detect: find first .onnx file in voices dir
    let dir = piper_voices_dir();
    if dir.is_dir() {
        if let Ok(entries) = std::fs::read_dir(&dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().map(|e| e == "onnx").unwrap_or(false)
                    && !path.to_string_lossy().ends_with(".onnx.json")
                {
                    return Some(path.to_string_lossy().to_string());
                }
            }
        }
    }

    None
}

/// Piper neural TTS implementation
pub struct PiperTts {
    model: Option<String>,
    noise_scale: f32,
    noise_w_scale: f32,
    length_scale: f32,
}

impl PiperTts {
    pub fn new(config: &TtsConfig) -> Self {
        Self {
            model: resolve_piper_model(&config.piper_model),
            noise_scale: config.piper_noise_scale.unwrap_or(0.4),
            noise_w_scale: config.piper_noise_w_scale.unwrap_or(0.2),
            length_scale: config.piper_length_scale.unwrap_or(1.05),
        }
    }

    #[allow(dead_code)]
    pub fn from_defaults() -> Self {
        Self {
            model: resolve_piper_model(&None),
            noise_scale: 0.4,
            noise_w_scale: 0.2,
            length_scale: 1.05,
        }
    }

    /// Piper is available only if the binary exists AND a model is found
    pub fn is_available() -> bool {
        which::which("piper").is_ok() && resolve_piper_model(&None).is_some()
    }

    /// Build piper args with model and voice tuning flags
    fn piper_args(&self, model: &str) -> Vec<String> {
        vec![
            "--model".to_string(), model.to_string(),
            "--output-raw".to_string(),
            "--noise-scale".to_string(), format!("{}", self.noise_scale),
            "--noise-w-scale".to_string(), format!("{}", self.noise_w_scale),
            "--length-scale".to_string(), format!("{}", self.length_scale),
        ]
    }
}

impl TextToSpeech for PiperTts {
    fn speak(&self, text: &str) -> Result<()> {
        let model = match &self.model {
            Some(m) => m.clone(),
            None => anyhow::bail!("No piper model found"),
        };
        let cleaned = format_text_for_speech(text);
        let args = self.piper_args(&model);

        std::thread::spawn(move || {
            let mut cmd = Command::new("piper");
            cmd.args(&args);
            cmd.stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::null());

            if let Ok(mut piper) = cmd.spawn() {
                if let Some(mut stdin) = piper.stdin.take() {
                    use std::io::Write;
                    let _ = stdin.write_all(cleaned.as_bytes());
                }
                if let Some(stdout) = piper.stdout.take() {
                    let _ = Command::new("aplay")
                        .args(["-r", "22050", "-f", "S16_LE", "-t", "raw", "-"])
                        .stdin(stdout)
                        .stdout(Stdio::null())
                        .stderr(Stdio::null())
                        .status();
                }
            }
        });

        Ok(())
    }

    fn speak_blocking(&self, text: &str) -> Result<()> {
        let model = match &self.model {
            Some(m) => m.clone(),
            None => anyhow::bail!("No piper model found in ~/.kgate/voices/piper/"),
        };
        let cleaned = format_text_for_speech(text);
        let args = self.piper_args(&model);

        let mut piper = Command::new("piper")
            .args(&args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()?;

        if let Some(mut stdin) = piper.stdin.take() {
            use std::io::Write;
            stdin.write_all(cleaned.as_bytes())?;
        }

        if let Some(stdout) = piper.stdout.take() {
            let aplay_status = Command::new("aplay")
                .args(["-r", "22050", "-f", "S16_LE", "-t", "raw", "-"])
                .stdin(stdout)
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status()?;
            if !aplay_status.success() {
                anyhow::bail!("aplay exited with {}", aplay_status);
            }
        }

        let piper_status = piper.wait()?;
        if !piper_status.success() {
            anyhow::bail!("piper exited with {}", piper_status);
        }
        Ok(())
    }
}

/// Kokoro neural TTS implementation (high quality, natural voices)
pub struct KokoroTts {
    voice: String,
}

impl KokoroTts {
    pub fn new(config: &TtsConfig) -> Self {
        Self {
            voice: config.kokoro_voice.clone().unwrap_or_else(|| "af_bella".to_string()),
        }
    }

    #[allow(dead_code)]
    pub fn from_defaults() -> Self {
        Self {
            voice: "af_bella".to_string(),
        }
    }

    pub fn is_available() -> bool {
        VoiceConfig::has_kokoro_python()
    }

    fn kokoro_script(&self) -> String {
        format!(
            r#"
import sys
from kokoro import KPipeline
import soundfile as sf
import tempfile
import subprocess

text = sys.stdin.read()
if not text.strip():
    sys.exit(0)

pipeline = KPipeline(lang_code='a')
audio_chunks = []
for _, _, audio in pipeline(text, voice='{}'):
    audio_chunks.append(audio)

if audio_chunks:
    import numpy as np
    full_audio = np.concatenate(audio_chunks)
    with tempfile.NamedTemporaryFile(suffix='.wav', delete=False) as f:
        sf.write(f.name, full_audio, 24000)
        subprocess.run(['aplay', '-q', f.name], check=True)
"#,
            self.voice
        )
    }
}

impl TextToSpeech for KokoroTts {
    fn speak(&self, text: &str) -> Result<()> {
        let cleaned = format_text_for_speech(text);
        let script = self.kokoro_script();

        std::thread::spawn(move || {
            let mut child = Command::new("python3")
                .args(["-c", &script])
                .stdin(Stdio::piped())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()
                .ok();

            if let Some(ref mut proc) = child {
                if let Some(mut stdin) = proc.stdin.take() {
                    use std::io::Write;
                    let _ = stdin.write_all(cleaned.as_bytes());
                }
                let _ = proc.wait();
            }
        });

        Ok(())
    }

    fn speak_blocking(&self, text: &str) -> Result<()> {
        let cleaned = format_text_for_speech(text);
        let script = self.kokoro_script();

        let mut child = Command::new("python3")
            .args(["-c", &script])
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()?;

        if let Some(mut stdin) = child.stdin.take() {
            use std::io::Write;
            stdin.write_all(cleaned.as_bytes())?;
        }

        let status = child.wait()?;
        if !status.success() {
            anyhow::bail!("kokoro python script exited with {}", status);
        }
        Ok(())
    }
}

/// Create a TTS instance based on engine config
pub fn create_tts(config: &TtsConfig) -> Box<dyn TextToSpeech + Send> {
    match config.engine {
        TtsEngine::Kokoro => Box::new(KokoroTts::new(config)),
        TtsEngine::Piper => Box::new(PiperTts::new(config)),
        TtsEngine::EspeakNg => Box::new(EspeakTts::new(config)),
    }
}

/// Check if a TTS engine actually works by running a quick smoke test.
/// Returns true only if the engine can successfully produce speech.
fn test_tts_engine(tts: &dyn TextToSpeech) -> bool {
    // Speak an empty-ish string that's fast but proves the engine runs
    tts.speak_blocking(".").is_ok()
}

/// Create a TTS instance with automatic fallback if configured engine isn't available.
///
/// Tries the configured engine first, then falls back through: kokoro -> piper -> espeak-ng.
/// Each candidate is smoke-tested by actually invoking it.
/// Returns None if no TTS engine is available at all.
pub fn create_tts_with_fallback(config: &TtsConfig) -> Option<Box<dyn TextToSpeech + Send>> {
    // Try configured engine first (check binary exists)
    let configured_available = match config.engine {
        TtsEngine::Kokoro => KokoroTts::is_available(),
        TtsEngine::Piper => PiperTts::is_available(),
        TtsEngine::EspeakNg => EspeakTts::is_available(),
    };

    if configured_available {
        let tts = create_tts(config);
        // Smoke test: actually try to speak
        if test_tts_engine(tts.as_ref()) {
            println!(
                "  {} TTS engine: {}",
                style("✓").green(),
                style(config.engine.to_string()).cyan()
            );
            return Some(tts);
        }
        println!(
            "  {} {} found but failed smoke test, trying fallback...",
            style("⚠").yellow(),
            config.engine
        );
    } else {
        println!(
            "  {} {} not found on PATH, trying fallback...",
            style("⚠").yellow(),
            config.engine
        );
    }

    // Fallback order: kokoro -> piper -> espeak-ng
    // Skip the configured engine (already tried above)
    let candidates: Vec<(&str, Box<dyn TextToSpeech + Send>)> = vec![
        ("kokoro", Box::new(KokoroTts::new(config))),
        ("piper", Box::new(PiperTts::new(config))),
        ("espeak-ng", Box::new(EspeakTts::new(config))),
    ];

    for (name, tts) in candidates {
        // Skip if this is the engine we already tried
        let is_configured = match config.engine {
            TtsEngine::Kokoro => name == "kokoro",
            TtsEngine::Piper => name == "piper",
            TtsEngine::EspeakNg => name == "espeak-ng",
        };
        if is_configured {
            continue;
        }

        // Quick binary check first (avoids slow python import for kokoro)
        let binary_exists = match name {
            "kokoro" => KokoroTts::is_available(),
            "piper" => PiperTts::is_available(),
            "espeak-ng" => EspeakTts::is_available(),
            _ => false,
        };

        if !binary_exists {
            continue;
        }

        if test_tts_engine(tts.as_ref()) {
            println!(
                "  {} Fallback TTS engine: {}",
                style("✓").green(),
                style(name).cyan()
            );
            return Some(tts);
        }
    }

    println!(
        "  {} No TTS engine available on PATH.",
        style("✗").red()
    );
    println!(
        "  {} Run kgate from nix-shell, or install: espeak-ng, piper, or kokoro",
        style("→").dim()
    );
    None
}

/// Clean text for TTS consumption
fn format_text_for_speech(text: &str) -> String {
    let mut result = text.to_string();

    // Remove markdown formatting
    result = result.replace("**", "");
    result = result.replace("__", "");
    result = result.replace("```", "");
    result = result.replace("`", "");
    result = result.replace("###", "");
    result = result.replace("##", "");
    result = result.replace("#", "");

    // Convert code-related symbols to speakable text
    result = result.replace("->", " returns ");
    result = result.replace("=>", " arrow ");
    result = result.replace("!=", " not equal to ");
    result = result.replace("==", " equals ");
    result = result.replace("&&", " and ");
    result = result.replace("||", " or ");
    result = result.replace("<=", " less than or equal to ");
    result = result.replace(">=", " greater than or equal to ");
    result = result.replace("&", " reference ");
    result = result.replace("*", " pointer ");

    // Strip any remaining letter option prefixes (A), B), etc.)
    result = result.replace("A) ", "");
    result = result.replace("B) ", "");
    result = result.replace("C) ", "");
    result = result.replace("D) ", "");

    // Clean up extra whitespace
    result = result.split_whitespace().collect::<Vec<_>>().join(" ");

    result
}

/// Format a question for speech (includes option handling)
pub fn format_question_for_speech(
    question_text: &str,
    options: &[String],
    code_snippet: Option<&str>,
) -> String {
    let mut parts = Vec::new();

    // Question text
    parts.push(format_text_for_speech(question_text));

    // Code snippet (simplified)
    if let Some(code) = code_snippet {
        let code_cleaned = code
            .lines()
            .filter(|l| !l.trim().starts_with("```"))
            .take(5) // Limit lines for voice
            .collect::<Vec<_>>()
            .join(". ");
        if !code_cleaned.is_empty() {
            parts.push(format!("Code: {}", format_text_for_speech(&code_cleaned)));
        }
    }

    // Options (numbered)
    for (i, opt) in options.iter().enumerate() {
        parts.push(format!("Option {}: {}", i + 1, format_text_for_speech(opt)));
    }

    parts.join(". ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_text_for_speech() {
        assert_eq!(
            format_text_for_speech("**bold** text"),
            "bold text"
        );
        assert_eq!(
            format_text_for_speech("a -> b"),
            "a returns b"
        );
        assert_eq!(
            format_text_for_speech("A) First B) Second"),
            "First Second"
        );
    }
}

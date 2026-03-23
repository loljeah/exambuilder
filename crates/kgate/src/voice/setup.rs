//! Voice setup wizard
//!
//! Guides user through configuring voice mode

use anyhow::Result;
use console::style;
use dialoguer::{theme::ColorfulTheme, Confirm, Input, Select};

use super::config::{CalibrationConfig, SttConfig, SttEngine, TtsConfig, TtsEngine, VoiceConfig};
use super::stt::SpeechToText;
use super::tts::create_tts_with_fallback;

/// Run the voice setup wizard
pub fn run_setup_wizard() -> Result<VoiceConfig> {
    println!();
    println!(
        "{}",
        style("Voice Mode Setup").cyan().bold()
    );
    println!("{}", style("─".repeat(40)).dim());
    println!();

    // Step 1: Detect TTS engines
    println!("{}", style("Step 1: Text-to-Speech").bold());
    let tts_engines = VoiceConfig::detect_tts_engines();

    if tts_engines.is_empty() {
        println!(
            "{} No TTS engines found. Start piper-daemon, or install espeak-ng/piper.",
            style("✗").red()
        );
        println!("  On NixOS: nix-shell -p espeak-ng");
        return Err(anyhow::anyhow!("No TTS engines available"));
    }

    println!(
        "  {} Found: {}",
        style("✓").green(),
        tts_engines
            .iter()
            .map(|e| e.to_string())
            .collect::<Vec<_>>()
            .join(", ")
    );

    // Select TTS engine
    let tts_engine = if tts_engines.len() == 1 {
        println!("  Using: {}", tts_engines[0]);
        tts_engines[0].clone()
    } else {
        let choices: Vec<String> = tts_engines.iter().map(|e| e.to_string()).collect();
        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Select TTS engine")
            .items(&choices)
            .default(0)
            .interact()?;
        tts_engines[selection].clone()
    };

    // TTS settings
    let voice: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Voice (e.g., en-us, en-gb)")
        .default("en-us".to_string())
        .interact_text()?;

    let speed: u32 = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Speech speed (words per minute)")
        .default(150)
        .interact_text()?;

    let piper_model = if tts_engine == TtsEngine::Piper {
        let model: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("Piper model path (or leave empty for default)")
            .default(String::new())
            .interact_text()?;
        if model.is_empty() {
            None
        } else {
            Some(model)
        }
    } else {
        // piper-daemon handles voice selection via its own tray/config
        None
    };

    // Get kokoro voice if using Kokoro engine
    let kokoro_voice = if tts_engine == TtsEngine::Kokoro {
        Some("af_bella".to_string())
    } else {
        None
    };

    let tts_config = TtsConfig {
        engine: tts_engine.clone(),
        voice: voice.clone(),
        speed,
        piper_model,
        kokoro_voice,
        piper_noise_scale: Some(0.4),
        piper_noise_w_scale: Some(0.2),
        piper_length_scale: Some(1.05),
    };

    // Test TTS
    println!();
    println!("{}", style("Testing TTS...").dim());

    let test_phrase = "Voice mode is ready. Let's learn something new.";
    match create_tts_with_fallback(&tts_config) {
        Some(tts) => {
            if let Err(e) = tts.speak_blocking(test_phrase) {
                println!("  {} TTS test failed: {}", style("⚠").yellow(), e);
            } else {
                println!("  {} TTS working", style("✓").green());
            }
        }
        None => {
            println!("  {} No TTS engine binary found on PATH", style("⚠").yellow());
        }
    }

    // Step 2: Detect STT engines
    println!();
    println!("{}", style("Step 2: Speech-to-Text").bold());
    let stt_engines = VoiceConfig::detect_stt_engines();

    let stt_config = if stt_engines.is_empty() {
        println!(
            "  {} No STT engines found. Voice input will be disabled.",
            style("⚠").yellow()
        );
        println!("  Install whisper.cpp for voice input support.");
        println!("  You can still use voice output with keyboard input.");

        SttConfig {
            engine: SttEngine::Whisper,
            model: "base.en".to_string(),
            language: "en".to_string(),
        }
    } else {
        println!(
            "  {} Found: {}",
            style("✓").green(),
            stt_engines
                .iter()
                .map(|e| e.to_string())
                .collect::<Vec<_>>()
                .join(", ")
        );

        let model: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("Whisper model (tiny.en, base.en, small.en)")
            .default("base.en".to_string())
            .interact_text()?;

        let language: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("Language")
            .default("en".to_string())
            .interact_text()?;

        SttConfig {
            engine: SttEngine::Whisper,
            model,
            language,
        }
    };

    // Step 3: Microphone setup
    println!();
    println!("{}", style("Step 3: Microphone").bold());

    let has_arecord = VoiceConfig::has_arecord();
    if !has_arecord {
        println!(
            "  {} arecord not found. Install alsa-utils.",
            style("⚠").yellow()
        );
    } else {
        println!("  {} arecord available", style("✓").green());
    }

    let mic_device: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Microphone device (leave empty for default)")
        .default(String::new())
        .interact_text()?;

    let sample_rate: u32 = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Sample rate (16000 recommended for whisper)")
        .default(16000)
        .interact_text()?;

    // Step 4: Calibration settings
    println!();
    println!("{}", style("Step 4: Calibration").bold());

    let max_wait: u32 = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Max wait time for answer (ms)")
        .default(5000)
        .interact_text()?;

    let confirm_answers = Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt("Confirm answers before submitting?")
        .default(true)
        .interact()?;

    let calibration = CalibrationConfig {
        silence_threshold: 0.01,
        max_wait_time_ms: max_wait,
        confirm_answers,
        microphone_device: if mic_device.is_empty() {
            None
        } else {
            Some(mic_device)
        },
        sample_rate,
    };

    // Build final config
    let config = VoiceConfig {
        general: super::config::GeneralConfig { enabled: true },
        tts: tts_config,
        stt: stt_config,
        calibration,
    };

    // Save config
    config.save()?;

    println!();
    println!(
        "{} Voice mode configured!",
        style("✓").green().bold()
    );
    println!(
        "  Config saved to: {}",
        VoiceConfig::config_path().display()
    );
    println!();
    println!("  Test with: kgate voice test-speak \"Hello world\"");
    println!("  Run exam:  kgate take exam 1 sprint 1 --voice");
    println!();

    Ok(config)
}

/// Test TTS with a specific phrase
pub fn test_speak(text: &str) -> Result<()> {
    let config = VoiceConfig::load()?;

    println!("Speaking: \"{}\"", text);

    match create_tts_with_fallback(&config.tts) {
        Some(tts) => {
            tts.speak_blocking(text)?;
            println!("{} Done", style("✓").green());
        }
        None => {
            println!(
                "{} No TTS engine available. Start piper-daemon, or install espeak-ng/piper/kokoro.",
                style("✗").red()
            );
        }
    }
    Ok(())
}

/// Test STT by recording and transcribing
pub fn test_listen() -> Result<()> {
    let config = VoiceConfig::load()?;

    let stt_available = !VoiceConfig::detect_stt_engines().is_empty();
    if !stt_available {
        println!(
            "{} No STT engine available. Install whisper.cpp.",
            style("✗").red()
        );
        return Ok(());
    }

    println!("Listening for 3 seconds... Speak now!");

    let stt = super::stt::WhisperStt::new(&config.stt, &config.calibration);

    match stt.listen_with_timeout(3000) {
        Ok(Some(text)) => {
            println!("{} Heard: \"{}\"", style("✓").green(), text);

            // Try to parse as answer
            if let Some(answer) = super::stt::parse_voice_answer(&text) {
                println!("  Parsed as answer: {}", style(answer).cyan().bold());
            }
        }
        Ok(None) => {
            println!("{} No speech detected", style("○").yellow());
        }
        Err(e) => {
            println!("{} Error: {}", style("✗").red(), e);
        }
    }

    Ok(())
}

/// Show current voice config
pub fn show_config() -> Result<()> {
    let path = VoiceConfig::config_path();

    if !path.exists() {
        println!("No voice config found.");
        println!("Run: kgate voice setup");
        return Ok(());
    }

    let config = VoiceConfig::load()?;

    println!("{}", style("Voice Configuration").cyan().bold());
    println!();
    println!("  Enabled: {}", config.general.enabled);
    println!();
    println!("  {}", style("TTS:").bold());
    println!("    Engine: {}", config.tts.engine);
    println!("    Voice:  {}", config.tts.voice);
    println!("    Speed:  {} wpm", config.tts.speed);
    if let Some(ref model) = config.tts.piper_model {
        println!("    Model:  {}", model);
    }
    println!();
    println!("  {}", style("STT:").bold());
    println!("    Engine:   {}", config.stt.engine);
    println!("    Model:    {}", config.stt.model);
    println!("    Language: {}", config.stt.language);
    println!();
    println!("  {}", style("Calibration:").bold());
    println!("    Max wait: {} ms", config.calibration.max_wait_time_ms);
    println!("    Confirm:  {}", config.calibration.confirm_answers);
    println!(
        "    Mic:      {}",
        config
            .calibration
            .microphone_device
            .as_deref()
            .unwrap_or("default")
    );
    println!();
    println!("  Config file: {}", path.display());

    Ok(())
}

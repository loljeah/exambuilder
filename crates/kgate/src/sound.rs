use std::io::Cursor;
use std::path::PathBuf;
use rodio::{Decoder, OutputStream, Sink};

// Embedded sound data (short sine wave tones generated procedurally)
// These are placeholder PCM data - will be replaced with actual sound files

#[allow(dead_code)]
pub fn sounds_dir() -> PathBuf {
    dirs::home_dir()
        .expect("No home directory")
        .join(".kgate")
        .join("sounds")
}

pub fn play_correct() {
    // Pleasant chime - high pitched short tone
    play_tone(880.0, 0.15); // A5
}

pub fn play_wrong() {
    // Low buzz
    play_tone(220.0, 0.2); // A3
}

#[allow(dead_code)]
pub fn play_levelup() {
    // Ascending fanfare
    std::thread::spawn(|| {
        play_tone(523.25, 0.1); // C5
        std::thread::sleep(std::time::Duration::from_millis(100));
        play_tone(659.25, 0.1); // E5
        std::thread::sleep(std::time::Duration::from_millis(100));
        play_tone(783.99, 0.1); // G5
        std::thread::sleep(std::time::Duration::from_millis(100));
        play_tone(1046.50, 0.2); // C6
    });
}

pub fn play_badge() {
    // Special achievement sound
    std::thread::spawn(|| {
        play_tone(659.25, 0.1); // E5
        std::thread::sleep(std::time::Duration::from_millis(80));
        play_tone(783.99, 0.15); // G5
    });
}

pub fn play_sprint_pass() {
    // Victory jingle
    std::thread::spawn(|| {
        play_tone(523.25, 0.1); // C5
        std::thread::sleep(std::time::Duration::from_millis(100));
        play_tone(659.25, 0.1); // E5
        std::thread::sleep(std::time::Duration::from_millis(100));
        play_tone(783.99, 0.2); // G5
    });
}

fn play_tone(freq: f32, duration_secs: f32) {
    // Generate a simple sine wave tone
    let sample_rate = 44100u32;
    let num_samples = (sample_rate as f32 * duration_secs) as usize;

    let mut samples: Vec<i16> = Vec::with_capacity(num_samples);

    for i in 0..num_samples {
        let t = i as f32 / sample_rate as f32;
        // Sine wave with envelope (fade in/out)
        let envelope = if i < num_samples / 10 {
            i as f32 / (num_samples / 10) as f32
        } else if i > num_samples * 9 / 10 {
            (num_samples - i) as f32 / (num_samples / 10) as f32
        } else {
            1.0
        };
        let sample = (envelope * 0.3 * (2.0 * std::f32::consts::PI * freq * t).sin() * i16::MAX as f32) as i16;
        samples.push(sample);
    }

    // Convert to WAV format in memory
    let mut wav_data = Vec::new();

    // WAV header
    wav_data.extend_from_slice(b"RIFF");
    let file_size = 36 + samples.len() * 2;
    wav_data.extend_from_slice(&(file_size as u32).to_le_bytes());
    wav_data.extend_from_slice(b"WAVE");
    wav_data.extend_from_slice(b"fmt ");
    wav_data.extend_from_slice(&16u32.to_le_bytes()); // chunk size
    wav_data.extend_from_slice(&1u16.to_le_bytes()); // PCM format
    wav_data.extend_from_slice(&1u16.to_le_bytes()); // mono
    wav_data.extend_from_slice(&sample_rate.to_le_bytes()); // sample rate
    wav_data.extend_from_slice(&(sample_rate * 2).to_le_bytes()); // byte rate
    wav_data.extend_from_slice(&2u16.to_le_bytes()); // block align
    wav_data.extend_from_slice(&16u16.to_le_bytes()); // bits per sample
    wav_data.extend_from_slice(b"data");
    wav_data.extend_from_slice(&(samples.len() as u32 * 2).to_le_bytes());

    for sample in samples {
        wav_data.extend_from_slice(&sample.to_le_bytes());
    }

    // Play the sound (spawn thread to not block)
    std::thread::spawn(move || {
        if let Ok((_stream, stream_handle)) = OutputStream::try_default() {
            if let Ok(sink) = Sink::try_new(&stream_handle) {
                let cursor = Cursor::new(wav_data);
                if let Ok(source) = Decoder::new(cursor) {
                    sink.append(source);
                    sink.sleep_until_end();
                }
            }
        }
    });
}

// Check if sounds are enabled in settings
#[allow(dead_code)]
pub fn is_enabled() -> bool {
    // TODO: read from database settings
    true
}

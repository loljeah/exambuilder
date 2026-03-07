# Future Features

## Walk Mode — Hands-Free Exam Taking

Take exams while walking around the flat, away from the computer.

**Concept:**
- Small handheld button (Bluetooth/ESP32)
- Press button → triggers Moonshine STT (external)
- Kokoro reads questions via speaker/earbuds
- Answer by voice, text appears in terminal
- No screen needed

**Components:**
- [ ] Bluetooth button/clicker (or ESP32 DIY)
- [ ] Moonshine integration (external, already working)
- [x] Kokoro TTS with natural female voice (done)
- [ ] Audio output to portable speaker/earbuds
- [ ] kgate daemon mode (listens for button events)

**Flow:**
1. Start walk mode: `kgate exam walk 1`
2. Kokoro reads question through earbuds
3. Press button → Moonshine listens
4. Speak answer (A/B/C/D)
5. Kokoro confirms: "You said B"
6. Press button to confirm / double-press to retry
7. Next question auto-reads

**Hardware ideas:**
- Xiaomi Mi Button
- ESP32 with single button + BLE
- Flic button
- Phone widget as fallback

---

## Voice Configuration

### TTS Engine: Kokoro-82M

Natural, warm female voices. Much better than robotic Piper/espeak.

**Best voices:**
| Voice | Traits | Grade |
|-------|--------|-------|
| `af_heart` | ❤️ Warm, natural | A |
| `af_bella` | 🔥 Expressive | A- |
| `af_nicole` | 🎧 Clear | B- |
| `bf_emma` | British | B- |

**Setup:**
```bash
pip install kokoro>=0.9.2 soundfile
apt install espeak-ng  # required for G2P
```

**Config (~/.kgate/voice.toml):**
```toml
[general]
enabled = true

[tts]
engine = "kokoro"
voice = "af_heart"
speed = 150
kokoro_voice = "af_heart"

[stt]
engine = "whisper"
model = "base.en"
language = "en"

[calibration]
silence_threshold = 0.01
max_wait_time_ms = 5000
confirm_answers = false
sample_rate = 16000
```

**Voice options:**
- `af_heart` — warm, natural (recommended)
- `af_bella` — slightly more expressive
- `bf_emma` — British accent
- Full list: https://huggingface.co/hexgrad/Kokoro-82M/blob/main/VOICES.md

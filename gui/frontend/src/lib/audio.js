// Audio feedback for exam interactions
// Uses Web Audio API to generate sound effects

let audioContext = null;

function getAudioContext() {
  if (!audioContext) {
    audioContext = new (window.AudioContext || window.webkitAudioContext)();
  }
  return audioContext;
}

// Play a simple tone
function playTone(frequency, duration, type = 'sine', volume = 0.3) {
  try {
    const ctx = getAudioContext();
    const oscillator = ctx.createOscillator();
    const gainNode = ctx.createGain();

    oscillator.connect(gainNode);
    gainNode.connect(ctx.destination);

    oscillator.type = type;
    oscillator.frequency.setValueAtTime(frequency, ctx.currentTime);

    gainNode.gain.setValueAtTime(volume, ctx.currentTime);
    gainNode.gain.exponentialRampToValueAtTime(0.01, ctx.currentTime + duration);

    oscillator.start(ctx.currentTime);
    oscillator.stop(ctx.currentTime + duration);
  } catch (e) {
    console.warn('Audio not available:', e);
  }
}

// Correct answer sound - happy ascending tones
export function playCorrect() {
  playTone(523.25, 0.1, 'sine', 0.2); // C5
  setTimeout(() => playTone(659.25, 0.1, 'sine', 0.2), 100); // E5
  setTimeout(() => playTone(783.99, 0.15, 'sine', 0.25), 200); // G5
}

// Wrong answer sound - descending buzz
export function playWrong() {
  playTone(200, 0.15, 'sawtooth', 0.15);
  setTimeout(() => playTone(150, 0.2, 'sawtooth', 0.1), 150);
}

// Sprint passed sound - victory fanfare
export function playSprintPassed() {
  const notes = [523, 659, 784, 1047]; // C5, E5, G5, C6
  notes.forEach((freq, i) => {
    setTimeout(() => playTone(freq, 0.2, 'sine', 0.2), i * 150);
  });
}

// Sprint failed sound - sad trombone
export function playSprintFailed() {
  const notes = [392, 370, 349, 330]; // G4 descending
  notes.forEach((freq, i) => {
    setTimeout(() => playTone(freq, 0.25, 'triangle', 0.15), i * 200);
  });
}

// Level up sound - epic ascending
export function playLevelUp() {
  const notes = [262, 330, 392, 523, 659, 784, 1047]; // C major scale up
  notes.forEach((freq, i) => {
    setTimeout(() => {
      playTone(freq, 0.12, 'sine', 0.2);
      playTone(freq * 1.5, 0.12, 'sine', 0.1); // Add fifth
    }, i * 80);
  });
}

// Achievement unlocked - magical chime
export function playAchievement() {
  playTone(880, 0.1, 'sine', 0.2);
  setTimeout(() => playTone(1047, 0.1, 'sine', 0.2), 100);
  setTimeout(() => playTone(1319, 0.15, 'sine', 0.25), 200);
  setTimeout(() => playTone(1760, 0.3, 'sine', 0.2), 300);
}

// Click/select sound
export function playClick() {
  playTone(800, 0.05, 'sine', 0.1);
}

// XP gain sound - coin-like
export function playXP() {
  playTone(1200, 0.08, 'square', 0.1);
  setTimeout(() => playTone(1500, 0.08, 'square', 0.08), 50);
}

<script>
  export let creature = 'cat';
  export let mood = 'neutral';
  export let hat = null;
  export let held = null;
  export let aura = null;
  export let size = 'medium'; // small, medium, large

  const sizes = {
    small: 48,
    medium: 64,
    large: 128,
  };

  $: sizePixels = sizes[size] || 64;

  // Mood to emoji (fallback until pixel art is loaded)
  const moodEmojis = {
    happy: '😊',
    content: '🙂',
    neutral: '😐',
    sad: '😢',
    lonely: '😔',
  };

  // Creature to emoji (fallback)
  const creatureEmojis = {
    cat: '🐱',
    slime: '🟢',
    octopus: '🐙',
    snail: '🐌',
  };

  $: moodEmoji = moodEmojis[mood] || '😐';
  $: creatureEmoji = creatureEmojis[creature] || '🐱';

  // Sprite paths (will be filled with actual pixel art)
  $: spritePath = `/sprites/${creature}/${mood}.png`;
  $: hatPath = hat ? `/sprites/hats/${hat}.png` : null;
  $: heldPath = held ? `/sprites/held/${held}.png` : null;
</script>

<div
  class="avatar-container"
  class:has-aura={aura}
  data-aura={aura}
  style="--size: {sizePixels}px"
>
  {#if aura}
    <div class="aura aura-{aura}"></div>
  {/if}

  <div class="avatar-sprite">
    <!-- Fallback emoji display until sprites are loaded -->
    <img
      src={spritePath}
      alt={creature}
      on:error={(e) => e.target.style.display = 'none'}
    />
    <span class="emoji-fallback">{creatureEmoji}</span>
  </div>

  {#if hat}
    <img class="accessory hat" src={hatPath} alt={hat} />
  {/if}

  {#if held}
    <img class="accessory held" src={heldPath} alt={held} />
  {/if}

  <div class="mood-indicator">
    <span>{moodEmoji}</span>
  </div>
</div>

<style>
  .avatar-container {
    position: relative;
    width: var(--size);
    height: var(--size);
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .avatar-sprite {
    width: 100%;
    height: 100%;
    display: flex;
    align-items: center;
    justify-content: center;
    image-rendering: pixelated;
    image-rendering: crisp-edges;
  }

  .avatar-sprite img {
    width: 100%;
    height: 100%;
    object-fit: contain;
  }

  .emoji-fallback {
    font-size: calc(var(--size) * 0.6);
    position: absolute;
    z-index: 1;
  }

  .avatar-sprite img + .emoji-fallback {
    display: none;
  }

  .avatar-sprite img:not([src]),
  .avatar-sprite img[style*="display: none"] + .emoji-fallback {
    display: block;
  }

  .accessory {
    position: absolute;
    image-rendering: pixelated;
    image-rendering: crisp-edges;
    width: 50%;
    height: 50%;
    object-fit: contain;
  }

  .accessory.hat {
    top: -10%;
    left: 50%;
    transform: translateX(-50%);
  }

  .accessory.held {
    bottom: 20%;
    right: -10%;
  }

  .mood-indicator {
    position: absolute;
    bottom: -4px;
    right: -4px;
    font-size: calc(var(--size) * 0.25);
    background: var(--bg-card);
    border-radius: 50%;
    padding: 2px;
    line-height: 1;
  }

  /* Aura effects */
  .aura {
    position: absolute;
    inset: -10%;
    border-radius: 50%;
    opacity: 0.3;
    animation: pulse 2s ease-in-out infinite;
    z-index: -1;
  }

  .aura-sparkles {
    background: radial-gradient(circle, var(--accent-gold) 0%, transparent 70%);
  }

  .aura-hearts {
    background: radial-gradient(circle, var(--accent-pink) 0%, transparent 70%);
  }

  .aura-stars {
    background: radial-gradient(circle, var(--primary-400) 0%, transparent 70%);
  }

  .aura-flames {
    background: radial-gradient(circle, var(--accent-red) 0%, transparent 70%);
  }

  .aura-rainbow {
    background: conic-gradient(
      red, orange, yellow, green, blue, purple, red
    );
    opacity: 0.2;
  }

  @keyframes pulse {
    0%, 100% { transform: scale(1); opacity: 0.3; }
    50% { transform: scale(1.1); opacity: 0.5; }
  }
</style>

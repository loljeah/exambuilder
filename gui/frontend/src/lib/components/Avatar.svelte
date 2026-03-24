<script>
  export let creature = 'cat';
  export let mood = 'neutral';
  export let hat = null;
  export let held = null;
  export let aura = null;
  export let background = null;
  export let size = 'medium'; // small, medium, large

  const sizes = {
    small: 48,
    medium: 64,
    large: 128,
  };

  $: sizePixels = sizes[size] || 64;

  // Creature to emoji (fallback if SVG fails)
  const creatureEmojis = {
    cat: '🐱',
    slime: '🟢',
    octopus: '🐙',
    snail: '🐌',
  };

  $: creatureEmoji = creatureEmojis[creature] || '🐱';

  // SVG sprite paths (vector art)
  $: spritePath = `/sprites/${creature}/${mood}.svg`;
  $: hatPath = hat ? `/sprites/hats/${hat}.svg` : null;
  $: heldPath = held ? `/sprites/held/${held}.svg` : null;
  $: auraPath = aura ? `/sprites/auras/${aura}.svg` : null;
  $: bgPath = background ? `/sprites/backgrounds/${background}.svg` : null;

  let spriteError = false;

  function handleSpriteError() {
    spriteError = true;
  }
</script>

<div
  class="avatar-container"
  class:has-aura={aura}
  class:has-background={background}
  style="--size: {sizePixels}px"
>
  {#if background}
    <img class="avatar-background" src={bgPath} alt="" />
  {/if}

  {#if aura}
    <img class="avatar-aura" src={auraPath} alt="" />
  {/if}

  <div class="avatar-sprite">
    {#if !spriteError}
      <img
        src={spritePath}
        alt={creature}
        on:error={handleSpriteError}
      />
    {:else}
      <span class="emoji-fallback">{creatureEmoji}</span>
    {/if}
  </div>

  {#if hat}
    <img class="accessory hat" src={hatPath} alt={hat} />
  {/if}

  {#if held}
    <img class="accessory held" src={heldPath} alt={held} />
  {/if}
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

  .avatar-background {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
    object-fit: cover;
    border-radius: var(--radius-md, 8px);
    z-index: 0;
  }

  .avatar-aura {
    position: absolute;
    inset: -20%;
    width: 140%;
    height: 140%;
    object-fit: contain;
    z-index: 1;
    pointer-events: none;
  }

  .avatar-sprite {
    position: relative;
    width: 80%;
    height: 80%;
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 2;
  }

  .avatar-sprite img {
    width: 100%;
    height: 100%;
    object-fit: contain;
  }

  .emoji-fallback {
    font-size: calc(var(--size) * 0.5);
    line-height: 1;
  }

  .accessory {
    position: absolute;
    width: 50%;
    height: 50%;
    object-fit: contain;
    z-index: 3;
    pointer-events: none;
  }

  .accessory.hat {
    top: -5%;
    left: 50%;
    transform: translateX(-50%);
  }

  .accessory.held {
    bottom: 10%;
    right: -5%;
    width: 40%;
    height: 40%;
  }

  /* Container with background gets rounded corners */
  .has-background {
    border-radius: var(--radius-md, 8px);
    overflow: hidden;
  }
</style>

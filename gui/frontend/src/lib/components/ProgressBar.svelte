<script>
  export let value = 0;
  export let max = 100;
  export let showLabel = false;
  export let showPercent = false;
  export let color = 'primary'; // primary, green, gold, red
  export let size = 'medium'; // small, medium, large

  $: percent = Math.min(100, Math.max(0, (value / max) * 100));
</script>

<div class="progress-container progress-{size}">
  <div class="progress-bar">
    <div
      class="progress-fill progress-{color}"
      style="width: {percent}%"
    ></div>
  </div>
  {#if showLabel || showPercent}
    <span class="progress-label">
      {#if showLabel}
        {value}/{max}
      {:else}
        {Math.round(percent)}%
      {/if}
    </span>
  {/if}
</div>

<style>
  .progress-container {
    display: flex;
    align-items: center;
    gap: var(--spacing-sm);
  }

  .progress-bar {
    flex: 1;
    background: var(--bg-tertiary);
    border-radius: var(--radius-sm);
    overflow: hidden;
  }

  .progress-small .progress-bar { height: 4px; }
  .progress-medium .progress-bar { height: 8px; }
  .progress-large .progress-bar { height: 12px; }

  .progress-fill {
    height: 100%;
    border-radius: var(--radius-sm);
    transition: width 0.3s ease;
  }

  .progress-primary { background: var(--primary-500); }
  .progress-green { background: var(--accent-green); }
  .progress-gold { background: var(--accent-gold); }
  .progress-red { background: var(--accent-red); }

  .progress-label {
    font-size: 12px;
    color: var(--text-muted);
    min-width: 40px;
    text-align: right;
  }
</style>

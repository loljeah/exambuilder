<script>
  export let icon = '';
  export let label = '';
  export let value = '';
  export let subtext = '';
  export let color = 'default'; // default, gold, green, red
</script>

<div class="stat-card stat-{color}">
  <span class="stat-icon">{icon}</span>
  <div class="stat-content">
    <span class="stat-value">{value}</span>
    <span class="stat-label">{label}</span>
    {#if subtext}
      <span class="stat-subtext">{subtext}</span>
    {/if}
  </div>
</div>

<style>
  .stat-card {
    display: flex;
    align-items: center;
    gap: var(--spacing-md);
    padding: var(--spacing-md);
    background: var(--bg-tertiary);
    border-radius: var(--radius-md);
  }

  .stat-icon {
    font-size: 24px;
    width: 40px;
    height: 40px;
    display: flex;
    align-items: center;
    justify-content: center;
    background: var(--bg-card);
    border-radius: var(--radius-md);
  }

  .stat-content {
    display: flex;
    flex-direction: column;
  }

  .stat-value {
    font-size: 18px;
    font-weight: 700;
    color: var(--text-primary);
  }

  .stat-gold .stat-value { color: var(--accent-gold); }
  .stat-green .stat-value { color: var(--accent-green); }
  .stat-red .stat-value { color: var(--accent-red); }

  .stat-label {
    font-size: 12px;
    color: var(--text-muted);
  }

  .stat-subtext {
    font-size: 11px;
    color: var(--text-muted);
    opacity: 0.7;
  }
</style>

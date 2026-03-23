<script>
  export let title = '';
  export let subtitle = '';
  export let padding = true;
</script>

<div class="card" class:no-padding={!padding}>
  {#if title || subtitle || $$slots.header}
    <div class="card-header">
      {#if $$slots.header}
        <slot name="header" />
      {:else}
        <h3 class="card-title">{title}</h3>
        {#if subtitle}
          <p class="card-subtitle">{subtitle}</p>
        {/if}
      {/if}
    </div>
  {/if}

  <div class="card-content">
    <slot />
  </div>

  {#if $$slots.footer}
    <div class="card-footer">
      <slot name="footer" />
    </div>
  {/if}
</div>

<style>
  .card {
    background: var(--bg-card);
    border-radius: var(--radius-lg);
    overflow: hidden;
    box-shadow: var(--shadow-sm);
  }

  .card-header {
    padding: var(--spacing-md) var(--spacing-lg);
    border-bottom: 1px solid var(--bg-tertiary);
  }

  .card-title {
    font-size: 16px;
    font-weight: 600;
    color: var(--text-primary);
    margin: 0;
  }

  .card-subtitle {
    font-size: 13px;
    color: var(--text-muted);
    margin: var(--spacing-xs) 0 0;
  }

  .card-content {
    padding: var(--spacing-lg);
  }

  .card.no-padding .card-content {
    padding: 0;
  }

  .card-footer {
    padding: var(--spacing-md) var(--spacing-lg);
    border-top: 1px solid var(--bg-tertiary);
    background: var(--bg-secondary);
  }
</style>

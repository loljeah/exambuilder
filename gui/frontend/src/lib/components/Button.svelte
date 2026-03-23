<script>
  export let variant = 'primary'; // primary, secondary, ghost, danger
  export let size = 'medium'; // small, medium, large
  export let disabled = false;
  export let loading = false;
  export let icon = null;
</script>

<button
  class="btn btn-{variant} btn-{size}"
  {disabled}
  class:loading
  on:click
>
  {#if loading}
    <span class="spinner"></span>
  {:else if icon}
    <span class="icon">{icon}</span>
  {/if}
  <slot />
</button>

<style>
  .btn {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: var(--spacing-sm);
    border: none;
    border-radius: var(--radius-md);
    font-weight: 500;
    cursor: pointer;
    transition: all 0.15s ease;
    font-family: inherit;
  }

  /* Sizes */
  .btn-small {
    padding: var(--spacing-xs) var(--spacing-sm);
    font-size: 12px;
  }

  .btn-medium {
    padding: var(--spacing-sm) var(--spacing-md);
    font-size: 14px;
  }

  .btn-large {
    padding: var(--spacing-md) var(--spacing-lg);
    font-size: 16px;
  }

  /* Variants */
  .btn-primary {
    background: var(--primary-600);
    color: white;
  }

  .btn-primary:hover:not(:disabled) {
    background: var(--primary-500);
  }

  .btn-secondary {
    background: var(--bg-tertiary);
    color: var(--text-primary);
  }

  .btn-secondary:hover:not(:disabled) {
    background: var(--bg-hover);
  }

  .btn-ghost {
    background: transparent;
    color: var(--text-secondary);
  }

  .btn-ghost:hover:not(:disabled) {
    background: var(--bg-tertiary);
    color: var(--text-primary);
  }

  .btn-danger {
    background: var(--accent-red);
    color: white;
  }

  .btn-danger:hover:not(:disabled) {
    background: #ef4444;
  }

  /* States */
  .btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .btn.loading {
    pointer-events: none;
  }

  .icon {
    font-size: 1.1em;
  }

  .spinner {
    width: 14px;
    height: 14px;
    border: 2px solid transparent;
    border-top-color: currentColor;
    border-radius: 50%;
    animation: spin 0.6s linear infinite;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }
</style>

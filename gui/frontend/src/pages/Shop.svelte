<script>
  import { onMount } from 'svelte';
  import Card from '../lib/components/Card.svelte';
  import Button from '../lib/components/Button.svelte';
  import { wallet } from '../lib/stores/wallet.js';

  let items = [];
  let filter = '';
  let loading = false;

  const rarityColors = {
    common: 'var(--rarity-common)',
    uncommon: 'var(--rarity-uncommon)',
    rare: 'var(--rarity-rare)',
    legendary: 'var(--rarity-legendary)',
  };

  async function loadItems() {
    if (window.go?.main?.App?.GetShopItems) {
      items = await window.go.main.App.GetShopItems(filter);
    }
  }

  async function buyItem(item) {
    if (window.go?.main?.App?.PurchaseItem) {
      loading = true;
      try {
        await window.go.main.App.PurchaseItem(item.id);
        wallet.spend(item.price);
        loadItems();
      } catch (err) {
        console.error('Purchase failed:', err);
      }
      loading = false;
    }
  }

  onMount(() => {
    loadItems();
    wallet.refresh();
  });

  $: groupedItems = items.reduce((acc, item) => {
    if (!acc[item.slot]) acc[item.slot] = [];
    acc[item.slot].push(item);
    return acc;
  }, {});
</script>

<div class="shop-page">
  <div class="shop-header">
    <h1 class="page-title">Shop</h1>
    <div class="wallet-display">
      💰 <span class="coins">{$wallet.coins}</span> coins
    </div>
  </div>

  <div class="filter-tabs">
    <button class:active={filter === ''} on:click={() => { filter = ''; loadItems(); }}>All</button>
    <button class:active={filter === 'hat'} on:click={() => { filter = 'hat'; loadItems(); }}>Hats</button>
    <button class:active={filter === 'held'} on:click={() => { filter = 'held'; loadItems(); }}>Held</button>
    <button class:active={filter === 'aura'} on:click={() => { filter = 'aura'; loadItems(); }}>Auras</button>
    <button class:active={filter === 'background'} on:click={() => { filter = 'background'; loadItems(); }}>Backgrounds</button>
  </div>

  <div class="shop-content">
    {#each Object.entries(groupedItems) as [slot, slotItems]}
      <Card title={slot.charAt(0).toUpperCase() + slot.slice(1) + 's'}>
        <div class="items-grid">
          {#each slotItems as item}
            <div class="shop-item" class:owned={item.owned} style="--rarity-color: {rarityColors[item.rarity]}">
              <div class="item-preview">
                <span class="item-icon">
                  {#if slot === 'hat'}🎩
                  {:else if slot === 'held'}📚
                  {:else if slot === 'aura'}✨
                  {:else}🏠
                  {/if}
                </span>
              </div>
              <div class="item-info">
                <span class="item-name">{item.name}</span>
                <span class="item-rarity">{item.rarity}</span>
              </div>
              <div class="item-footer">
                {#if item.owned}
                  <span class="owned-badge">Owned</span>
                {:else}
                  <span class="item-price">{item.price} 🪙</span>
                  <Button
                    size="small"
                    disabled={$wallet.coins < item.price || loading}
                    on:click={() => buyItem(item)}
                  >
                    Buy
                  </Button>
                {/if}
              </div>
            </div>
          {/each}
        </div>
      </Card>
    {:else}
      <Card>
        <p class="empty">Loading shop items...</p>
      </Card>
    {/each}
  </div>
</div>

<style>
  .shop-page {
    max-width: 1200px;
    margin: 0 auto;
  }

  .shop-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: var(--spacing-lg);
  }

  .page-title {
    font-size: 24px;
    font-weight: 700;
    margin: 0;
  }

  .wallet-display {
    font-size: 18px;
    padding: var(--spacing-sm) var(--spacing-md);
    background: var(--bg-card);
    border-radius: var(--radius-md);
  }

  .coins {
    color: var(--accent-gold);
    font-weight: 700;
  }

  .filter-tabs {
    display: flex;
    gap: var(--spacing-xs);
    margin-bottom: var(--spacing-lg);
  }

  .filter-tabs button {
    padding: var(--spacing-sm) var(--spacing-md);
    background: var(--bg-card);
    border: none;
    border-radius: var(--radius-md);
    color: var(--text-secondary);
    cursor: pointer;
    transition: all 0.15s;
  }

  .filter-tabs button:hover {
    background: var(--bg-tertiary);
  }

  .filter-tabs button.active {
    background: var(--primary-600);
    color: white;
  }

  .shop-content {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-lg);
  }

  .items-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(140px, 1fr));
    gap: var(--spacing-md);
  }

  .shop-item {
    background: var(--bg-tertiary);
    border-radius: var(--radius-md);
    padding: var(--spacing-md);
    text-align: center;
    border: 2px solid transparent;
    transition: all 0.15s;
  }

  .shop-item:hover {
    border-color: var(--rarity-color);
  }

  .shop-item.owned {
    opacity: 0.6;
  }

  .item-preview {
    width: 48px;
    height: 48px;
    margin: 0 auto var(--spacing-sm);
    background: var(--bg-card);
    border-radius: var(--radius-md);
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 24px;
  }

  .item-name {
    display: block;
    font-weight: 600;
    font-size: 13px;
  }

  .item-rarity {
    font-size: 11px;
    color: var(--rarity-color);
    text-transform: capitalize;
  }

  .item-footer {
    margin-top: var(--spacing-sm);
    display: flex;
    flex-direction: column;
    gap: var(--spacing-xs);
    align-items: center;
  }

  .item-price {
    color: var(--accent-gold);
    font-weight: 600;
  }

  .owned-badge {
    color: var(--accent-green);
    font-size: 12px;
  }

  .empty {
    text-align: center;
    color: var(--text-muted);
    padding: var(--spacing-xl);
  }
</style>

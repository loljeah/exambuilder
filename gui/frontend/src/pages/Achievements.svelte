<script>
  import { onMount } from 'svelte';
  import Card from '../lib/components/Card.svelte';
  import ProgressBar from '../lib/components/ProgressBar.svelte';

  let achievements = [];
  let unlocked = 0;
  let total = 0;
  let filter = 'all';

  async function loadAchievements() {
    if (window.go?.main?.App?.GetAchievements) {
      achievements = await window.go.main.App.GetAchievements();
    }
    if (window.go?.main?.App?.GetAchievementCounts) {
      [unlocked, total] = await window.go.main.App.GetAchievementCounts();
    }
  }

  onMount(loadAchievements);

  const hiddenCategories = ['collection'];

  $: filteredAchievements = achievements.filter(a => {
    if (hiddenCategories.includes(a.category)) return false;
    if (filter === 'all') return true;
    if (filter === 'unlocked') return a.unlocked;
    if (filter === 'locked') return !a.unlocked;
    return a.category === filter;
  });

  $: groupedAchievements = filteredAchievements.reduce((acc, a) => {
    if (!acc[a.category]) acc[a.category] = [];
    acc[a.category].push(a);
    return acc;
  }, {});
</script>

<div class="achievements-page">
  <div class="achievements-header">
    <h1 class="page-title">Achievements</h1>
    <div class="achievement-count">
      <span class="unlocked">{unlocked}</span>
      <span class="separator">/</span>
      <span class="total">{total}</span>
      <span class="label">unlocked</span>
    </div>
  </div>

  <div class="filter-tabs">
    <button class:active={filter === 'all'} on:click={() => filter = 'all'}>All</button>
    <button class:active={filter === 'unlocked'} on:click={() => filter = 'unlocked'}>Unlocked</button>
    <button class:active={filter === 'locked'} on:click={() => filter = 'locked'}>Locked</button>
  </div>

  <div class="achievements-content">
    {#each Object.entries(groupedAchievements) as [category, categoryAchievements]}
      <Card title={category.charAt(0).toUpperCase() + category.slice(1)}>
        <div class="achievement-list">
          {#each categoryAchievements as achievement}
            <div class="achievement" class:unlocked={achievement.unlocked} class:secret={achievement.secret && !achievement.unlocked}>
              <div class="achievement-icon">
                {#if achievement.unlocked}
                  {achievement.icon}
                {:else if achievement.secret}
                  🔒
                {:else}
                  🔒
                {/if}
              </div>
              <div class="achievement-info">
                <h4>
                  {#if achievement.secret && !achievement.unlocked}
                    ???
                  {:else}
                    {achievement.name}
                  {/if}
                </h4>
                <p>
                  {#if achievement.secret && !achievement.unlocked}
                    Secret achievement
                  {:else}
                    {achievement.description}
                  {/if}
                </p>
                {#if achievement.unlocked}
                  <span class="unlocked-date">Unlocked: {achievement.unlocked_at || 'Recently'}</span>
                {/if}
              </div>
              <div class="achievement-reward">
                <span class="coins">+{achievement.reward_coins}</span>
                <span class="coin-icon">🪙</span>
              </div>
            </div>
          {/each}
        </div>
      </Card>
    {:else}
      <Card>
        <p class="empty">No achievements found</p>
      </Card>
    {/each}
  </div>
</div>

<style>
  .achievements-page {
    max-width: 900px;
    margin: 0 auto;
  }

  .achievements-header {
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

  .achievement-count {
    display: flex;
    align-items: baseline;
    gap: 4px;
    font-size: 18px;
  }

  .achievement-count .unlocked {
    font-weight: 700;
    color: var(--accent-green);
  }

  .achievement-count .separator {
    color: var(--text-muted);
  }

  .achievement-count .total {
    color: var(--text-secondary);
  }

  .achievement-count .label {
    font-size: 12px;
    color: var(--text-muted);
    margin-left: 4px;
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

  .achievements-content {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-lg);
  }

  .achievement-list {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-sm);
  }

  .achievement {
    display: flex;
    align-items: center;
    gap: var(--spacing-md);
    padding: var(--spacing-md);
    background: var(--bg-tertiary);
    border-radius: var(--radius-md);
    opacity: 0.6;
  }

  .achievement.unlocked {
    opacity: 1;
    background: linear-gradient(135deg, var(--bg-tertiary) 0%, var(--primary-900) 100%);
  }

  .achievement.secret {
    opacity: 0.4;
  }

  .achievement-icon {
    width: 48px;
    height: 48px;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 24px;
    background: var(--bg-card);
    border-radius: var(--radius-md);
  }

  .achievement.unlocked .achievement-icon {
    background: var(--primary-600);
  }

  .achievement-info {
    flex: 1;
  }

  .achievement-info h4 {
    margin: 0 0 var(--spacing-xs);
    font-size: 15px;
  }

  .achievement-info p {
    margin: 0;
    font-size: 13px;
    color: var(--text-muted);
  }

  .unlocked-date {
    font-size: 11px;
    color: var(--accent-green);
  }

  .achievement-reward {
    display: flex;
    align-items: center;
    gap: 4px;
    color: var(--accent-gold);
  }

  .coins {
    font-weight: 600;
  }

  .empty {
    text-align: center;
    color: var(--text-muted);
    padding: var(--spacing-xl);
  }
</style>

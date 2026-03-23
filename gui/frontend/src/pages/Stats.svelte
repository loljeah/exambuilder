<script>
  import Card from '../lib/components/Card.svelte';
  import StatCard from '../lib/components/StatCard.svelte';

  let period = 'week';
  let stats = [];

  // TODO: Load stats from backend and implement Chart.js charts
</script>

<div class="stats-page">
  <h1 class="page-title">Statistics</h1>

  <div class="period-selector">
    <button class:active={period === 'today'} on:click={() => period = 'today'}>Today</button>
    <button class:active={period === 'week'} on:click={() => period = 'week'}>Week</button>
    <button class:active={period === 'month'} on:click={() => period = 'month'}>Month</button>
  </div>

  <div class="stats-grid">
    <Card title="XP Over Time">
      <div class="chart-placeholder">
        <p>📈 XP chart coming soon</p>
      </div>
    </Card>

    <div class="stats-row">
      <StatCard icon="📝" label="Sprints" value="0 / 0" />
      <StatCard icon="🎯" label="Accuracy" value="0%" />
      <StatCard icon="⏱️" label="Time Spent" value="0m" />
    </div>

    <Card title="Streaks & Consistency">
      <div class="streak-info">
        <div class="streak-current">
          <span class="streak-icon">🔥</span>
          <span class="streak-value">0 days</span>
          <span class="streak-label">Current Streak</span>
        </div>
        <div class="streak-best">
          <span class="streak-icon">🏆</span>
          <span class="streak-value">0 days</span>
          <span class="streak-label">Best Streak</span>
        </div>
      </div>
    </Card>

    <Card title="Hardest Questions">
      <p class="empty">No question stats yet</p>
    </Card>
  </div>
</div>

<style>
  .stats-page {
    max-width: 1200px;
    margin: 0 auto;
  }

  .page-title {
    font-size: 24px;
    font-weight: 700;
    margin-bottom: var(--spacing-lg);
  }

  .period-selector {
    display: flex;
    gap: var(--spacing-xs);
    margin-bottom: var(--spacing-lg);
  }

  .period-selector button {
    padding: var(--spacing-sm) var(--spacing-md);
    background: var(--bg-card);
    border: none;
    border-radius: var(--radius-md);
    color: var(--text-secondary);
    cursor: pointer;
    transition: all 0.15s;
  }

  .period-selector button:hover {
    background: var(--bg-tertiary);
  }

  .period-selector button.active {
    background: var(--primary-600);
    color: white;
  }

  .stats-grid {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-lg);
  }

  .stats-row {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: var(--spacing-md);
  }

  .chart-placeholder {
    height: 200px;
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--text-muted);
    background: var(--bg-tertiary);
    border-radius: var(--radius-md);
  }

  .streak-info {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: var(--spacing-lg);
    text-align: center;
  }

  .streak-icon {
    font-size: 32px;
    display: block;
    margin-bottom: var(--spacing-sm);
  }

  .streak-value {
    display: block;
    font-size: 24px;
    font-weight: 700;
  }

  .streak-label {
    font-size: 12px;
    color: var(--text-muted);
  }

  .empty {
    text-align: center;
    color: var(--text-muted);
    padding: var(--spacing-xl);
  }
</style>

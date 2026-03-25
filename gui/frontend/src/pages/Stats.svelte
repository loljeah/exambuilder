<script>
  import { onMount } from 'svelte';
  import Card from '../lib/components/Card.svelte';
  import StatCard from '../lib/components/StatCard.svelte';
  import { dashboard } from '../lib/stores/dashboard.js';

  let period = 'week';
  let stats = [];
  let profile = null;

  async function loadStats() {
    if (window.go?.main?.App?.GetStats) {
      stats = await window.go.main.App.GetStats(period) || [];
    }
    if (window.go?.main?.App?.GetProfile) {
      profile = await window.go.main.App.GetProfile();
    }
  }

  function formatTime(seconds) {
    if (!seconds) return '0m';
    const hrs = Math.floor(seconds / 3600);
    const mins = Math.floor((seconds % 3600) / 60);
    if (hrs > 0) return `${hrs}h ${mins}m`;
    return `${mins}m`;
  }

  // Calculate aggregates from stats
  $: totalXP = stats.reduce((sum, s) => sum + (s.xp_earned || 0), 0);
  $: totalSprints = stats.reduce((sum, s) => sum + (s.sprints_passed || 0), 0);
  $: totalTime = stats.reduce((sum, s) => sum + (s.time_spent || 0), 0);
  $: totalCorrect = stats.reduce((sum, s) => sum + (s.questions_correct || 0), 0);
  $: totalQuestions = stats.reduce((sum, s) => sum + (s.questions_answered || 0), 0);
  $: accuracy = totalQuestions > 0 ? Math.round((totalCorrect / totalQuestions) * 100) : 0;

  // Calculate max for chart scaling
  $: maxXP = Math.max(...stats.map(s => s.xp_earned || 0), 1);

  onMount(() => {
    loadStats();
    dashboard.refresh();
  });

  $: {
    // Reload when period changes
    if (period) loadStats();
  }
</script>

<div class="stats-page">
  <h1 class="page-title">Statistics</h1>

  <div class="period-selector">
    <button class:active={period === 'today'} on:click={() => period = 'today'}>Today</button>
    <button class:active={period === 'week'} on:click={() => period = 'week'}>Week</button>
    <button class:active={period === 'month'} on:click={() => period = 'month'}>Month</button>
  </div>

  <div class="stats-grid">
    <!-- Summary Cards -->
    <div class="stats-row">
      <Card>
        <div class="stat-card-inner">
          <span class="stat-icon">✨</span>
          <span class="stat-value">{totalXP}</span>
          <span class="stat-label">XP Earned</span>
        </div>
      </Card>
      <Card>
        <div class="stat-card-inner">
          <span class="stat-icon">📝</span>
          <span class="stat-value">{totalSprints}</span>
          <span class="stat-label">Sprints Passed</span>
        </div>
      </Card>
      <Card>
        <div class="stat-card-inner">
          <span class="stat-icon">🎯</span>
          <span class="stat-value">{accuracy}%</span>
          <span class="stat-label">Accuracy</span>
        </div>
      </Card>
      <Card>
        <div class="stat-card-inner">
          <span class="stat-icon">⏱️</span>
          <span class="stat-value">{formatTime(totalTime)}</span>
          <span class="stat-label">Time Studied</span>
        </div>
      </Card>
    </div>

    <!-- XP Chart -->
    <Card title="XP Progress">
      {#if stats.length > 0}
        <div class="chart-container">
          <div class="chart-bars">
            {#each stats as day, i}
              <div class="chart-bar-container">
                <div
                  class="chart-bar"
                  style="height: {((day.xp_earned || 0) / maxXP) * 100}%"
                  title="{day.date}: {day.xp_earned || 0} XP"
                >
                  {#if day.xp_earned > 0}
                    <span class="bar-value">{day.xp_earned}</span>
                  {/if}
                </div>
                <span class="chart-label">
                  {period === 'today' ? 'Today' : day.date?.slice(-5) || `D${i+1}`}
                </span>
              </div>
            {/each}
          </div>
        </div>
      {:else}
        <div class="chart-placeholder">
          <p>📈 No data for this period</p>
        </div>
      {/if}
    </Card>

    <!-- Streaks -->
    <Card title="Streaks & Consistency">
      <div class="streak-info">
        <div class="streak-current">
          <span class="streak-icon">🔥</span>
          <span class="streak-value">{profile?.current_streak || $dashboard.profile.current_streak || 0}</span>
          <span class="streak-unit">days</span>
          <span class="streak-label">Current Streak</span>
        </div>
        <div class="streak-best">
          <span class="streak-icon">🏆</span>
          <span class="streak-value">{profile?.best_streak || $dashboard.profile.best_streak || 0}</span>
          <span class="streak-unit">days</span>
          <span class="streak-label">Best Streak</span>
        </div>
      </div>
    </Card>

    <!-- Activity Heatmap placeholder -->
    <Card title="Activity">
      <div class="activity-summary">
        <div class="activity-stat">
          <span class="activity-value">{totalQuestions}</span>
          <span class="activity-label">Questions Answered</span>
        </div>
        <div class="activity-stat">
          <span class="activity-value">{totalCorrect}</span>
          <span class="activity-label">Correct Answers</span>
        </div>
        <div class="activity-stat">
          <span class="activity-value">{profile?.sprints_passed || $dashboard.profile.sprints_passed || 0}</span>
          <span class="activity-label">Total Sprints Passed</span>
        </div>
        <div class="activity-stat">
          <span class="activity-value">Lv.{profile?.level || $dashboard.profile.level || 1}</span>
          <span class="activity-label">Current Level</span>
        </div>
      </div>
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
    padding: var(--spacing-sm) var(--spacing-lg);
    background: var(--bg-card);
    border: none;
    border-radius: var(--radius-lg);
    color: var(--text-secondary);
    cursor: pointer;
    transition: all 0.2s;
    font-weight: 500;
  }

  .period-selector button:hover {
    background: var(--bg-tertiary);
    transform: translateY(-1px);
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
    grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
    gap: var(--spacing-md);
  }

  .stat-card-inner {
    text-align: center;
    padding: var(--spacing-md);
  }

  .stat-icon {
    font-size: 32px;
    display: block;
    margin-bottom: var(--spacing-sm);
  }

  .stat-value {
    font-size: 28px;
    font-weight: 700;
    color: var(--text-primary);
    display: block;
  }

  .stat-label {
    font-size: 12px;
    color: var(--text-muted);
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  /* Chart */
  .chart-container {
    padding: var(--spacing-md) 0;
  }

  .chart-bars {
    display: flex;
    align-items: flex-end;
    justify-content: space-around;
    height: 200px;
    gap: var(--spacing-xs);
  }

  .chart-bar-container {
    flex: 1;
    max-width: 60px;
    display: flex;
    flex-direction: column;
    align-items: center;
    height: 100%;
  }

  .chart-bar {
    width: 100%;
    background: linear-gradient(180deg, var(--primary-400), var(--primary-600));
    border-radius: var(--radius-sm) var(--radius-sm) 0 0;
    min-height: 4px;
    position: relative;
    display: flex;
    align-items: flex-start;
    justify-content: center;
    transition: height 0.3s ease;
  }

  .bar-value {
    position: absolute;
    top: -20px;
    font-size: 11px;
    font-weight: 600;
    color: var(--text-secondary);
  }

  .chart-label {
    font-size: 10px;
    color: var(--text-muted);
    margin-top: var(--spacing-xs);
    text-align: center;
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

  /* Streaks */
  .streak-info {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: var(--spacing-lg);
    text-align: center;
    padding: var(--spacing-md);
  }

  .streak-current, .streak-best {
    padding: var(--spacing-lg);
    background: var(--bg-tertiary);
    border-radius: var(--radius-lg);
  }

  .streak-icon {
    font-size: 40px;
    display: block;
    margin-bottom: var(--spacing-sm);
  }

  .streak-value {
    display: inline;
    font-size: 36px;
    font-weight: 700;
    color: var(--text-primary);
  }

  .streak-unit {
    font-size: 16px;
    color: var(--text-secondary);
    margin-left: var(--spacing-xs);
  }

  .streak-label {
    display: block;
    font-size: 12px;
    color: var(--text-muted);
    margin-top: var(--spacing-xs);
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  /* Activity */
  .activity-summary {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(150px, 1fr));
    gap: var(--spacing-md);
    padding: var(--spacing-md);
  }

  .activity-stat {
    text-align: center;
    padding: var(--spacing-md);
    background: var(--bg-tertiary);
    border-radius: var(--radius-md);
  }

  .activity-value {
    display: block;
    font-size: 24px;
    font-weight: 700;
    color: var(--primary-400);
    margin-bottom: var(--spacing-xs);
  }

  .activity-label {
    font-size: 11px;
    color: var(--text-muted);
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  @media (max-width: 600px) {
    .stats-row {
      grid-template-columns: repeat(2, 1fr);
    }

    .streak-info {
      grid-template-columns: 1fr;
    }
  }
</style>

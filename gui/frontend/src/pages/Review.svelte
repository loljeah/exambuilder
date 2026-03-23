<script>
  import Card from '../lib/components/Card.svelte';
  import Button from '../lib/components/Button.svelte';
  import ProgressBar from '../lib/components/ProgressBar.svelte';

  let reviewItems = [];
  let masteryStats = { total: 0, mastered: 0, learning: 0, unseen: 0 };

  // TODO: Load from backend
</script>

<div class="review-page">
  <h1 class="page-title">Review</h1>

  <div class="review-grid">
    <Card title="Due Today" subtitle="Knowledge items ready for review">
      <div class="review-list">
        <p class="empty">No items due for review</p>
      </div>
      <svelte:fragment slot="footer">
        <Button>Start Review Session</Button>
      </svelte:fragment>
    </Card>

    <Card title="Mastery Overview">
      <div class="mastery-stats">
        <div class="stat-row">
          <span>Total concepts</span>
          <span>{masteryStats.total}</span>
        </div>
        <div class="stat-row mastered">
          <span>Mastered</span>
          <span>{masteryStats.mastered}</span>
        </div>
        <div class="stat-row learning">
          <span>Learning</span>
          <span>{masteryStats.learning}</span>
        </div>
        <div class="stat-row unseen">
          <span>Unseen</span>
          <span>{masteryStats.unseen}</span>
        </div>
      </div>
      {#if masteryStats.total > 0}
        <div class="mastery-bar">
          <div class="segment mastered" style="width: {(masteryStats.mastered / masteryStats.total) * 100}%"></div>
          <div class="segment learning" style="width: {(masteryStats.learning / masteryStats.total) * 100}%"></div>
          <div class="segment unseen" style="width: {(masteryStats.unseen / masteryStats.total) * 100}%"></div>
        </div>
      {/if}
    </Card>
  </div>
</div>

<style>
  .review-page {
    max-width: 1000px;
    margin: 0 auto;
  }

  .page-title {
    font-size: 24px;
    font-weight: 700;
    margin-bottom: var(--spacing-lg);
  }

  .review-grid {
    display: grid;
    grid-template-columns: 2fr 1fr;
    gap: var(--spacing-lg);
  }

  .review-list {
    min-height: 200px;
  }

  .empty {
    text-align: center;
    color: var(--text-muted);
    padding: var(--spacing-xl);
  }

  .mastery-stats {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-sm);
    margin-bottom: var(--spacing-md);
  }

  .stat-row {
    display: flex;
    justify-content: space-between;
    padding: var(--spacing-xs) 0;
    border-bottom: 1px solid var(--bg-tertiary);
  }

  .stat-row.mastered span:last-child { color: var(--accent-green); }
  .stat-row.learning span:last-child { color: var(--primary-400); }
  .stat-row.unseen span:last-child { color: var(--text-muted); }

  .mastery-bar {
    display: flex;
    height: 12px;
    border-radius: var(--radius-sm);
    overflow: hidden;
    background: var(--bg-tertiary);
  }

  .segment {
    height: 100%;
  }

  .segment.mastered { background: var(--accent-green); }
  .segment.learning { background: var(--primary-500); }
  .segment.unseen { background: var(--bg-tertiary); }
</style>

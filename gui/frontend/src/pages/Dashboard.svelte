<script>
  import { dashboard } from '../lib/stores/dashboard.js';
  import Card from '../lib/components/Card.svelte';
  import Avatar from '../lib/components/Avatar.svelte';
  import Button from '../lib/components/Button.svelte';
  import ProgressBar from '../lib/components/ProgressBar.svelte';
  import StatCard from '../lib/components/StatCard.svelte';

  $: data = $dashboard;
  $: xpToNextLevel = (data.profile.level + 1) * 100;
  $: xpProgress = data.profile.total_xp % 100;
  $: xpBonus = Math.round((data.avatar.xp_multiplier - 1) * 100);

  async function claimDaily() {
    if (window.go?.main?.App?.ClaimDailyReward) {
      const coins = await window.go.main.App.ClaimDailyReward();
      dashboard.refresh();
    }
  }

  async function claimChallenge(id) {
    if (window.go?.main?.App?.ClaimChallengeReward) {
      await window.go.main.App.ClaimChallengeReward(id);
      dashboard.refresh();
    }
  }
</script>

<div class="dashboard">
  <h1 class="page-title">Dashboard</h1>

  <div class="dashboard-grid">
    <!-- Avatar Card -->
    <Card title="Your Companion">
      <div class="avatar-section">
        <Avatar
          creature={data.avatar.creature_type}
          mood={data.avatar.mood}
          size="large"
        />
        <div class="avatar-details">
          <h3>{data.avatar.name}</h3>
          <p class="mood">
            {#if data.avatar.mood === 'happy'}
              😊 Happy
            {:else if data.avatar.mood === 'content'}
              🙂 Content
            {:else if data.avatar.mood === 'neutral'}
              😐 Neutral
            {:else if data.avatar.mood === 'sad'}
              😢 Sad
            {:else}
              😔 Lonely
            {/if}
          </p>
          {#if xpBonus > 0}
            <p class="xp-bonus">+{xpBonus}% XP bonus</p>
          {/if}
        </div>
      </div>
    </Card>

    <!-- Quick Stats -->
    <Card title="Quick Stats">
      <div class="stats-grid">
        <StatCard icon="⭐" label="Level" value={data.profile.level} />
        <StatCard icon="✨" label="Total XP" value={data.profile.total_xp} />
        <StatCard icon="🔥" label="Streak" value="{data.profile.current_streak} days" color="red" />
        <StatCard icon="💰" label="Coins" value={data.wallet.coins} color="gold" />
      </div>
      <div class="xp-progress">
        <span>XP to Level {data.profile.level + 1}</span>
        <ProgressBar value={xpProgress} max={100} showLabel color="primary" />
      </div>
    </Card>

    <!-- Daily Login -->
    <Card title="Daily Reward">
      <div class="daily-section">
        <div class="daily-info">
          <span class="day-label">Day {data.daily_login.current_day || 1} of 7</span>
          <span class="total-claims">{data.daily_login.total_claims} total claims</span>
        </div>
        <div class="daily-rewards">
          {#each [10, 15, 25, 40, 60, 85, 120] as reward, i}
            <div
              class="reward-day"
              class:claimed={i + 1 < data.daily_login.current_day}
              class:current={i + 1 === data.daily_login.current_day}
            >
              <span class="day">D{i + 1}</span>
              <span class="coins">{reward}</span>
            </div>
          {/each}
        </div>
        {#if data.daily_login.can_claim}
          <Button on:click={claimDaily}>Claim Daily Reward</Button>
        {:else}
          <Button disabled>Already Claimed Today</Button>
        {/if}
      </div>
    </Card>

    <!-- Daily Challenges -->
    <Card title="Daily Challenges">
      <div class="challenges">
        {#each data.challenges as challenge}
          <div class="challenge" class:completed={challenge.completed} class:claimed={challenge.claimed}>
            <div class="challenge-info">
              <span class="challenge-icon">
                {challenge.completed ? (challenge.claimed ? '💰' : '✓') : '○'}
              </span>
              <span class="challenge-desc">{challenge.description}</span>
            </div>
            <div class="challenge-progress">
              <ProgressBar
                value={challenge.progress}
                max={challenge.target}
                size="small"
                color={challenge.completed ? 'green' : 'primary'}
              />
              <span class="challenge-reward">{challenge.reward_coins} 🪙</span>
            </div>
            {#if challenge.completed && !challenge.claimed}
              <Button size="small" on:click={() => claimChallenge(challenge.id)}>
                Claim
              </Button>
            {/if}
          </div>
        {:else}
          <p class="empty">No challenges today</p>
        {/each}
      </div>
    </Card>

    <!-- Weekly Goals -->
    <Card title="Weekly Goals">
      <div class="goals">
        {#each data.weekly_goals as goal}
          <div class="goal" class:completed={goal.completed}>
            <div class="goal-header">
              <span>{goal.completed ? '✓' : '○'}</span>
              <span class="goal-desc">{goal.description}</span>
              <span class="goal-reward">{goal.reward_coins} 🪙</span>
            </div>
            <ProgressBar
              value={goal.progress}
              max={goal.target}
              showLabel
              size="small"
              color={goal.completed ? 'green' : 'primary'}
            />
          </div>
        {:else}
          <p class="empty">No weekly goals</p>
        {/each}
      </div>
    </Card>

    <!-- Active Project -->
    <Card title="Active Project">
      {#if data.active_project}
        <div class="project-info">
          <span class="project-name">📁 {data.active_project.name}</span>
          <span class="pending">{data.pending_sprints} sprints pending</span>
        </div>
        <Button variant="secondary">Continue Learning</Button>
      {:else}
        <p class="empty">No project selected</p>
        <Button variant="secondary">Select Project</Button>
      {/if}
    </Card>

    <!-- Review Due -->
    <Card title="Review Items">
      <div class="review-info">
        <span class="review-count">{data.review_due}</span>
        <span class="review-label">items due for review</span>
      </div>
      {#if data.review_due > 0}
        <Button>Start Review Session</Button>
      {:else}
        <p class="all-clear">All caught up! 🎉</p>
      {/if}
    </Card>
  </div>
</div>

<style>
  .dashboard {
    max-width: 1400px;
    margin: 0 auto;
  }

  .page-title {
    font-size: 24px;
    font-weight: 700;
    margin-bottom: var(--spacing-lg);
    color: var(--text-primary);
  }

  .dashboard-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
    gap: var(--spacing-lg);
  }

  /* Avatar Section */
  .avatar-section {
    display: flex;
    align-items: center;
    gap: var(--spacing-lg);
  }

  .avatar-details h3 {
    font-size: 18px;
    margin-bottom: var(--spacing-xs);
  }

  .avatar-details .mood {
    color: var(--text-secondary);
    margin-bottom: var(--spacing-xs);
  }

  .avatar-details .xp-bonus {
    color: var(--accent-green);
    font-weight: 600;
  }

  /* Stats */
  .stats-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: var(--spacing-sm);
    margin-bottom: var(--spacing-md);
  }

  .xp-progress {
    padding-top: var(--spacing-md);
    border-top: 1px solid var(--bg-tertiary);
  }

  .xp-progress span {
    display: block;
    font-size: 12px;
    color: var(--text-muted);
    margin-bottom: var(--spacing-xs);
  }

  /* Daily */
  .daily-section {
    text-align: center;
  }

  .daily-info {
    margin-bottom: var(--spacing-md);
  }

  .day-label {
    display: block;
    font-size: 18px;
    font-weight: 600;
  }

  .total-claims {
    font-size: 12px;
    color: var(--text-muted);
  }

  .daily-rewards {
    display: flex;
    justify-content: center;
    gap: var(--spacing-xs);
    margin-bottom: var(--spacing-md);
  }

  .reward-day {
    display: flex;
    flex-direction: column;
    align-items: center;
    padding: var(--spacing-xs);
    background: var(--bg-tertiary);
    border-radius: var(--radius-sm);
    min-width: 36px;
  }

  .reward-day.claimed {
    background: var(--accent-green);
    color: white;
  }

  .reward-day.current {
    background: var(--primary-600);
    color: white;
  }

  .reward-day .day {
    font-size: 10px;
  }

  .reward-day .coins {
    font-size: 12px;
    font-weight: 600;
  }

  /* Challenges */
  .challenges {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-sm);
  }

  .challenge {
    padding: var(--spacing-sm);
    background: var(--bg-tertiary);
    border-radius: var(--radius-md);
  }

  .challenge-info {
    display: flex;
    align-items: center;
    gap: var(--spacing-sm);
    margin-bottom: var(--spacing-xs);
  }

  .challenge-icon {
    font-size: 16px;
  }

  .challenge.completed .challenge-icon {
    color: var(--accent-green);
  }

  .challenge-progress {
    display: flex;
    align-items: center;
    gap: var(--spacing-sm);
  }

  .challenge-reward {
    font-size: 12px;
    color: var(--accent-gold);
  }

  /* Goals */
  .goals {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-md);
  }

  .goal-header {
    display: flex;
    align-items: center;
    gap: var(--spacing-sm);
    margin-bottom: var(--spacing-xs);
  }

  .goal-desc {
    flex: 1;
  }

  .goal-reward {
    color: var(--accent-gold);
    font-size: 12px;
  }

  .goal.completed {
    opacity: 0.7;
  }

  /* Project */
  .project-info {
    margin-bottom: var(--spacing-md);
  }

  .project-name {
    display: block;
    font-weight: 600;
  }

  .pending {
    font-size: 12px;
    color: var(--text-muted);
  }

  /* Review */
  .review-info {
    text-align: center;
    margin-bottom: var(--spacing-md);
  }

  .review-count {
    display: block;
    font-size: 36px;
    font-weight: 700;
    color: var(--primary-400);
  }

  .review-label {
    color: var(--text-muted);
  }

  .all-clear {
    text-align: center;
    color: var(--accent-green);
  }

  .empty {
    color: var(--text-muted);
    text-align: center;
    padding: var(--spacing-md);
  }
</style>

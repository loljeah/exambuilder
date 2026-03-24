<script>
  import { onMount } from 'svelte';
  import Card from '../lib/components/Card.svelte';
  import Button from '../lib/components/Button.svelte';

  let questions = [];
  let domains = [];
  let selectedDomain = '';
  let selectedTier = '';
  let searchQuery = '';
  let showMasteredOnly = false;
  let expandedQuestion = null;

  async function loadData() {
    if (window.go?.main?.App?.GetDomains) {
      domains = await window.go.main.App.GetDomains() || [];
    }
    if (window.go?.main?.App?.GetKnowledgeBase) {
      questions = await window.go.main.App.GetKnowledgeBase() || [];
    }
  }

  function filterQuestions(questions, domain, tier, search, masteredOnly) {
    return questions.filter(q => {
      if (domain && q.domain_id !== domain) return false;
      if (tier && q.tier !== tier) return false;
      if (masteredOnly && !q.mastered) return false;
      if (search) {
        const lower = search.toLowerCase();
        if (!q.text.toLowerCase().includes(lower) &&
            !q.sprint_topic.toLowerCase().includes(lower)) {
          return false;
        }
      }
      return true;
    });
  }

  function toggleQuestion(idx) {
    expandedQuestion = expandedQuestion === idx ? null : idx;
  }

  function getDifficultyClass(diff) {
    if (diff === 1) return 'easy';
    if (diff === 2) return 'medium';
    return 'hard';
  }

  function getTierColor(tier) {
    const colors = {
      'RECALL': '#22c55e',
      'COMPREHENSION': '#3b82f6',
      'APPLICATION': '#a855f7',
      'ANALYSIS': '#ef4444'
    };
    return colors[tier] || '#666';
  }

  $: filteredQuestions = filterQuestions(questions, selectedDomain, selectedTier, searchQuery, showMasteredOnly);
  $: questionsByDomain = filteredQuestions.reduce((acc, q) => {
    const key = q.domain_name || 'General';
    if (!acc[key]) acc[key] = [];
    acc[key].push(q);
    return acc;
  }, {});

  onMount(loadData);
</script>

<div class="knowledge-page">
  <h1 class="page-title">Knowledge Base</h1>
  <p class="page-subtitle">All questions organized by domain and topic</p>

  <!-- Filters -->
  <div class="filters">
    <div class="filter-group">
      <label>Domain</label>
      <select bind:value={selectedDomain}>
        <option value="">All Domains</option>
        {#each domains as domain}
          <option value={domain.domain_id}>{domain.icon} {domain.name}</option>
        {/each}
      </select>
    </div>

    <div class="filter-group">
      <label>Tier</label>
      <select bind:value={selectedTier}>
        <option value="">All Tiers</option>
        <option value="RECALL">Recall</option>
        <option value="COMPREHENSION">Comprehension</option>
        <option value="APPLICATION">Application</option>
        <option value="ANALYSIS">Analysis</option>
      </select>
    </div>

    <div class="filter-group search">
      <label>Search</label>
      <input type="text" bind:value={searchQuery} placeholder="Search questions..." />
    </div>

    <div class="filter-group toggle">
      <label>
        <input type="checkbox" bind:checked={showMasteredOnly} />
        Mastered only
      </label>
    </div>
  </div>

  <!-- Stats Summary -->
  <div class="stats-summary">
    <div class="stat">
      <span class="stat-value">{questions.length}</span>
      <span class="stat-label">Total Questions</span>
    </div>
    <div class="stat">
      <span class="stat-value">{questions.filter(q => q.mastered).length}</span>
      <span class="stat-label">Mastered</span>
    </div>
    <div class="stat">
      <span class="stat-value">{filteredQuestions.length}</span>
      <span class="stat-label">Showing</span>
    </div>
  </div>

  <!-- Questions by Domain -->
  {#each Object.entries(questionsByDomain) as [domainName, domainQuestions]}
    <Card title={domainName} subtitle="{domainQuestions.length} questions">
      <div class="questions-list">
        {#each domainQuestions as q, idx}
          {@const uniqueIdx = `${q.sprint_number}-${q.question_num}`}
          <div class="question-item" class:expanded={expandedQuestion === uniqueIdx}>
            <button class="question-header" on:click={() => toggleQuestion(uniqueIdx)}>
              <div class="question-meta">
                <span class="sprint-badge">S{q.sprint_number}</span>
                <span class="tier-badge" style="background: {getTierColor(q.tier)}">{q.tier}</span>
                <span class="difficulty-badge {getDifficultyClass(q.difficulty)}">
                  {'★'.repeat(q.difficulty)}
                </span>
                {#if q.mastered}
                  <span class="mastered-badge">✓</span>
                {/if}
              </div>
              <div class="question-preview">
                {q.text.length > 100 ? q.text.slice(0, 100) + '...' : q.text}
              </div>
              <span class="expand-icon">{expandedQuestion === uniqueIdx ? '▼' : '▶'}</span>
            </button>

            {#if expandedQuestion === uniqueIdx}
              <div class="question-details">
                <div class="question-text">
                  <p>{q.text}</p>
                  {#if q.code}
                    <pre class="code-block"><code>{q.code}</code></pre>
                  {/if}
                </div>

                <div class="options-list">
                  {#each q.options as option, optIdx}
                    <div class="option" class:correct={optIdx === q.correct_idx}>
                      <span class="option-letter">{String.fromCharCode(65 + optIdx)}</span>
                      <span class="option-text">{option}</span>
                      {#if optIdx === q.correct_idx}
                        <span class="correct-mark">✓</span>
                      {/if}
                    </div>
                  {/each}
                </div>

                {#if q.hint}
                  <div class="hint-box">
                    <strong>Hint:</strong> {q.hint}
                  </div>
                {/if}

                {#if q.explanation}
                  <div class="explanation-box">
                    <strong>Explanation:</strong> {q.explanation}
                  </div>
                {/if}

                <div class="question-footer">
                  <span class="topic-tag">{q.sprint_topic}</span>
                  <span class="xp-value">+{q.xp} XP</span>
                </div>
              </div>
            {/if}
          </div>
        {/each}
      </div>
    </Card>
  {:else}
    <Card>
      <p class="empty">No questions found matching your filters.</p>
    </Card>
  {/each}
</div>

<style>
  .knowledge-page {
    max-width: 900px;
    margin: 0 auto;
  }

  .page-title {
    font-size: 24px;
    font-weight: 700;
    margin: 0;
  }

  .page-subtitle {
    color: var(--text-muted);
    margin: var(--spacing-xs) 0 var(--spacing-lg);
  }

  /* Filters */
  .filters {
    display: flex;
    flex-wrap: wrap;
    gap: var(--spacing-md);
    margin-bottom: var(--spacing-lg);
    padding: var(--spacing-md);
    background: var(--bg-card);
    border-radius: var(--radius-md);
  }

  .filter-group {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-xs);
  }

  .filter-group label {
    font-size: 12px;
    color: var(--text-muted);
    font-weight: 500;
  }

  .filter-group select, .filter-group input[type="text"] {
    padding: var(--spacing-sm) var(--spacing-md);
    background: var(--bg-tertiary);
    border: 1px solid var(--bg-hover);
    border-radius: var(--radius-sm);
    color: var(--text-primary);
    min-width: 150px;
  }

  .filter-group.search {
    flex: 1;
  }

  .filter-group.search input {
    width: 100%;
  }

  .filter-group.toggle {
    justify-content: flex-end;
  }

  .filter-group.toggle label {
    display: flex;
    align-items: center;
    gap: var(--spacing-xs);
    cursor: pointer;
    font-size: 13px;
  }

  /* Stats Summary */
  .stats-summary {
    display: flex;
    gap: var(--spacing-lg);
    margin-bottom: var(--spacing-lg);
    padding: var(--spacing-md);
    background: var(--bg-card);
    border-radius: var(--radius-md);
  }

  .stat {
    display: flex;
    flex-direction: column;
    align-items: center;
  }

  .stat-value {
    font-size: 24px;
    font-weight: 700;
    color: var(--primary-400);
  }

  .stat-label {
    font-size: 12px;
    color: var(--text-muted);
  }

  /* Questions List */
  .questions-list {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-sm);
  }

  .question-item {
    border: 1px solid var(--bg-tertiary);
    border-radius: var(--radius-md);
    overflow: hidden;
  }

  .question-header {
    width: 100%;
    display: flex;
    align-items: center;
    gap: var(--spacing-md);
    padding: var(--spacing-md);
    background: var(--bg-tertiary);
    border: none;
    cursor: pointer;
    text-align: left;
    color: var(--text-primary);
    transition: background 0.15s;
  }

  .question-header:hover {
    background: var(--bg-hover);
  }

  .question-meta {
    display: flex;
    align-items: center;
    gap: var(--spacing-xs);
    flex-shrink: 0;
  }

  .sprint-badge {
    font-size: 11px;
    padding: 2px 6px;
    background: var(--bg-card);
    border-radius: var(--radius-sm);
    color: var(--text-muted);
  }

  .tier-badge {
    font-size: 10px;
    padding: 2px 6px;
    border-radius: var(--radius-sm);
    color: white;
    font-weight: 600;
  }

  .difficulty-badge {
    font-size: 11px;
  }

  .difficulty-badge.easy { color: var(--accent-green); }
  .difficulty-badge.medium { color: var(--accent-gold); }
  .difficulty-badge.hard { color: var(--accent-red); }

  .mastered-badge {
    width: 18px;
    height: 18px;
    display: flex;
    align-items: center;
    justify-content: center;
    background: var(--accent-green);
    border-radius: 50%;
    color: white;
    font-size: 11px;
  }

  .question-preview {
    flex: 1;
    font-size: 13px;
    color: var(--text-secondary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .expand-icon {
    color: var(--text-muted);
    font-size: 12px;
  }

  /* Question Details */
  .question-details {
    padding: var(--spacing-md);
    background: var(--bg-card);
    border-top: 1px solid var(--bg-tertiary);
  }

  .question-text {
    margin-bottom: var(--spacing-md);
  }

  .question-text p {
    margin: 0;
    line-height: 1.6;
  }

  .code-block {
    margin-top: var(--spacing-sm);
    padding: var(--spacing-md);
    background: var(--bg-primary);
    border-radius: var(--radius-sm);
    overflow-x: auto;
    font-family: 'JetBrains Mono', monospace;
    font-size: 12px;
  }

  .options-list {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-xs);
    margin-bottom: var(--spacing-md);
  }

  .option {
    display: flex;
    align-items: center;
    gap: var(--spacing-sm);
    padding: var(--spacing-sm) var(--spacing-md);
    background: var(--bg-tertiary);
    border-radius: var(--radius-sm);
    font-size: 13px;
  }

  .option.correct {
    background: rgba(34, 197, 94, 0.1);
    border: 1px solid var(--accent-green);
  }

  .option-letter {
    width: 22px;
    height: 22px;
    display: flex;
    align-items: center;
    justify-content: center;
    background: var(--bg-card);
    border-radius: 50%;
    font-weight: 600;
    font-size: 11px;
  }

  .option.correct .option-letter {
    background: var(--accent-green);
    color: white;
  }

  .option-text {
    flex: 1;
  }

  .correct-mark {
    color: var(--accent-green);
    font-weight: 700;
  }

  .hint-box, .explanation-box {
    padding: var(--spacing-sm) var(--spacing-md);
    border-radius: var(--radius-sm);
    font-size: 13px;
    margin-bottom: var(--spacing-sm);
  }

  .hint-box {
    background: rgba(252, 211, 77, 0.1);
    border-left: 3px solid var(--accent-gold);
  }

  .explanation-box {
    background: rgba(147, 112, 219, 0.1);
    border-left: 3px solid var(--primary-400);
  }

  .question-footer {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding-top: var(--spacing-sm);
    border-top: 1px solid var(--bg-tertiary);
  }

  .topic-tag {
    font-size: 12px;
    padding: 2px 8px;
    background: var(--bg-tertiary);
    border-radius: var(--radius-sm);
    color: var(--text-muted);
  }

  .xp-value {
    color: var(--accent-gold);
    font-weight: 600;
    font-size: 13px;
  }

  .empty {
    text-align: center;
    color: var(--text-muted);
    padding: var(--spacing-xl);
  }

  @media (max-width: 600px) {
    .filters {
      flex-direction: column;
    }

    .filter-group {
      width: 100%;
    }

    .stats-summary {
      justify-content: space-around;
    }
  }
</style>

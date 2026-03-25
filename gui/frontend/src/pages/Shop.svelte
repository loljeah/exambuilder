<script>
  import { onMount } from 'svelte';
  import Card from '../lib/components/Card.svelte';
  import Button from '../lib/components/Button.svelte';
  import { wallet } from '../lib/stores/wallet.js';

  // View: 'hints' | 'forge'
  let view = 'hints';
  let hintBalance = { tokens: 0, lifetime_tokens: 0 };
  let packs = [];
  let loading = false;
  let notification = '';

  // Knowledge Forge state
  let ollamaAvailable = false;
  let ollamaModels = [];
  let domains = [];
  let domainGates = {};
  let generating = false;
  let generationTopic = '';

  async function loadHintBalance() {
    if (window.go?.main?.App?.GetHintTokenBalance) {
      hintBalance = await window.go.main.App.GetHintTokenBalance() || { tokens: 0, lifetime_tokens: 0 };
    }
  }

  async function loadPacks() {
    if (window.go?.main?.App?.GetHintPacks) {
      packs = await window.go.main.App.GetHintPacks() || [];
    }
  }

  async function checkOllama() {
    if (window.go?.main?.App?.IsOllamaAvailable) {
      ollamaAvailable = await window.go.main.App.IsOllamaAvailable();
    }
    if (ollamaAvailable && window.go?.main?.App?.GetOllamaModels) {
      ollamaModels = await window.go.main.App.GetOllamaModels() || [];
    }
  }

  async function loadDomains() {
    if (window.go?.main?.App?.GetDomains) {
      domains = await window.go.main.App.GetDomains() || [];
      // Load generation gates for each domain
      for (const d of domains) {
        if (window.go?.main?.App?.GetGenerationGate) {
          domainGates[d.id] = await window.go.main.App.GetGenerationGate(d.id);
        }
      }
      domainGates = domainGates; // Trigger reactivity
    }
  }

  async function generateSprint(domainID) {
    if (!window.go?.main?.App?.GenerateSprintForDomain) return;
    const topic = generationTopic || domains.find(d => d.id === domainID)?.name || domainID;
    generating = true;
    try {
      const result = await window.go.main.App.GenerateSprintForDomain(domainID, topic);
      if (result.coins_spent === 0) {
        showNotification('Free sprint generated! Go take it!');
      } else {
        showNotification(`Sprint generated! (-${result.coins_spent} coins)`);
      }
      wallet.refresh();
      await loadDomains();
      generationTopic = '';
    } catch (err) {
      showNotification('Generation failed: ' + (err.message || err));
    }
    generating = false;
  }

  async function generateExam(domainID) {
    if (!window.go?.main?.App?.GenerateExamForDomain) return;
    generating = true;
    try {
      const result = await window.go.main.App.GenerateExamForDomain(domainID);
      showNotification(`Exam generated (3 sprints)! (-${result.coins_spent} coins)`);
      wallet.refresh();
      await loadDomains();
    } catch (err) {
      showNotification('Generation failed: ' + (err.message || err));
    }
    generating = false;
  }

  function costLabel(gate, type) {
    if (type === 'sprint' && !gate.first_free_used) return 'FREE';
    const costs = { sprint: gate.sprint_cost, custom: gate.custom_cost, exam: gate.exam_cost, challenge: gate.challenge_cost };
    return `${costs[type]} coins`;
  }

  async function purchasePack(tier) {
    if (!window.go?.main?.App?.PurchaseHintTokens) return;
    loading = true;
    try {
      await window.go.main.App.PurchaseHintTokens(tier);
      await loadHintBalance();
      wallet.refresh();
      const pack = packs.find(p => p.tier === tier);
      showNotification(`Purchased ${pack?.tokens || ''} hint tokens!`);
    } catch (err) {
      showNotification('Purchase failed: ' + (err.message || err));
    }
    loading = false;
  }

  function showNotification(msg) {
    notification = msg;
    setTimeout(() => notification = '', 3000);
  }

  function tierLabel(tier) {
    if (tier === 'small') return 'Starter';
    if (tier === 'medium') return 'Value';
    if (tier === 'large') return 'Bulk';
    return tier;
  }

  function tierBadge(tier) {
    if (tier === 'medium') return 'Best Value';
    if (tier === 'large') return 'Best Price';
    return '';
  }

  onMount(async () => {
    await Promise.all([loadHintBalance(), loadPacks(), wallet.refresh(), checkOllama(), loadDomains()]);
  });
</script>

<div class="store-page">
  <div class="store-header">
    <div class="header-left">
      <h1 class="page-title">Store</h1>
      <div class="view-toggle">
        <button class:active={view === 'hints'} on:click={() => view = 'hints'}>
          Hint Tokens
        </button>
        <button class:active={view === 'forge'} on:click={() => view = 'forge'}>
          Knowledge Forge
        </button>
      </div>
    </div>
    <div class="header-stats">
      <div class="wallet-display">
        💰 <span class="coins">{$wallet.coins}</span>
      </div>
      <div class="token-display">
        💡 <span class="tokens">{hintBalance.tokens}</span>
      </div>
    </div>
  </div>

  {#if notification}
    <div class="notification">{notification}</div>
  {/if}

  {#if view === 'hints'}
    <div class="hint-section">
      <Card title="Your Hint Tokens">
        <div class="token-overview">
          <div class="token-big">
            <span class="token-count">{hintBalance.tokens}</span>
            <span class="token-label">tokens available</span>
          </div>
          <div class="token-lifetime">
            {hintBalance.lifetime_tokens} lifetime tokens purchased
          </div>
        </div>
      </Card>

      <h2 class="section-title">Purchase Hint Tokens</h2>
      <p class="section-desc">Use hint tokens during exams to reveal hints before answering</p>

      <div class="packs-grid">
        {#each packs as pack}
          <div class="pack-card">
            {#if tierBadge(pack.tier)}
              <div class="pack-badge">{tierBadge(pack.tier)}</div>
            {/if}
            <div class="pack-icon">💡</div>
            <h3 class="pack-name">{tierLabel(pack.tier)} Pack</h3>
            <div class="pack-amount">{pack.tokens} hints</div>
            <div class="pack-per">
              {Math.round(pack.cost / pack.tokens)} coins each
            </div>
            <div class="pack-footer">
              <span class="pack-price">💰 {pack.cost}</span>
              <Button
                size="small"
                variant="primary"
                disabled={$wallet.coins < pack.cost || loading}
                on:click={() => purchasePack(pack.tier)}
              >
                Buy
              </Button>
            </div>
          </div>
        {:else}
          <p class="empty-state">Loading packs...</p>
        {/each}
      </div>

      <Card title="How Hint Tokens Work">
        <div class="how-it-works">
          <div class="step">
            <span class="step-num">1</span>
            <span>Purchase hint token packs above</span>
          </div>
          <div class="step">
            <span class="step-num">2</span>
            <span>During an exam, click "Use Hint" on any question</span>
          </div>
          <div class="step">
            <span class="step-num">3</span>
            <span>The hint is revealed — helping you answer correctly</span>
          </div>
          <div class="step">
            <span class="step-num">4</span>
            <span>Hints don't affect your score — they're a paid resource you earned</span>
          </div>
        </div>
      </Card>
    </div>

  {:else}
    <div class="forge-section">
      <Card title="Knowledge Forge">
        <div class="forge-status">
          <div class="ollama-status" class:available={ollamaAvailable}>
            <span class="status-dot"></span>
            <span>Ollama: {ollamaAvailable ? 'Connected' : 'Offline'}</span>
          </div>
          {#if !ollamaAvailable}
            <p class="forge-info">
              Knowledge Forge uses a local LLM to generate new exam content.
              Start Ollama to unlock this feature.
            </p>
          {:else}
            <p class="forge-info">
              Spend coins to generate new sprints and exams. Domain levels unlock more types.
            </p>
            {#if ollamaModels.length > 0}
              <div class="model-info">
                Model: <strong>{ollamaModels[0]}</strong>
                {#if ollamaModels.length > 1}
                  (+{ollamaModels.length - 1} more)
                {/if}
              </div>
            {/if}
          {/if}
        </div>
      </Card>

      {#if ollamaAvailable}
        <h2 class="section-title">Your Domains</h2>
        <p class="section-desc">Level up domains by passing sprints to unlock generation</p>

        {#if generating}
          <Card>
            <div class="generating-overlay">
              <div class="spinner"></div>
              <span>Generating content with LLM...</span>
              <span class="generating-sub">This may take 30-60 seconds</span>
            </div>
          </Card>
        {/if}

        <div class="domain-cards">
          {#each domains as domain}
            {@const gate = domainGates[domain.id] || {}}
            <Card>
              <div class="domain-card">
                <div class="domain-header">
                  <span class="domain-icon">{domain.icon || '📘'}</span>
                  <div class="domain-info">
                    <h3 class="domain-name">{domain.name}</h3>
                    <span class="domain-level">Level {gate.domain_level || 0}</span>
                  </div>
                </div>

                <div class="generation-types">
                  <!-- Sprint -->
                  <div class="gen-type" class:locked={!gate.can_sprint}>
                    <div class="gen-type-header">
                      <span class="gen-lock">{gate.can_sprint ? '🔓' : '🔒'}</span>
                      <span class="gen-name">Sprint (3Q)</span>
                      <span class="gen-unlock">Lv.3</span>
                    </div>
                    {#if gate.can_sprint}
                      <div class="gen-action">
                        <input
                          type="text"
                          class="topic-input"
                          placeholder="Topic (optional)"
                          bind:value={generationTopic}
                          disabled={generating}
                        />
                        <Button
                          size="small"
                          variant={!gate.first_free_used ? 'primary' : 'secondary'}
                          disabled={generating}
                          on:click={() => generateSprint(domain.id)}
                        >
                          {#if !gate.first_free_used}
                            FREE
                          {:else}
                            💰 {gate.sprint_cost}
                          {/if}
                        </Button>
                      </div>
                      {#if !gate.first_free_used}
                        <span class="free-badge">First generation free!</span>
                      {/if}
                    {/if}
                  </div>

                  <!-- Full Exam -->
                  <div class="gen-type" class:locked={!gate.can_exam}>
                    <div class="gen-type-header">
                      <span class="gen-lock">{gate.can_exam ? '🔓' : '🔒'}</span>
                      <span class="gen-name">Full Exam (3 sprints)</span>
                      <span class="gen-unlock">Lv.8</span>
                    </div>
                    {#if gate.can_exam}
                      <div class="gen-action">
                        <Button
                          size="small"
                          variant="secondary"
                          disabled={generating}
                          on:click={() => generateExam(domain.id)}
                        >
                          💰 {gate.exam_cost}
                        </Button>
                      </div>
                    {/if}
                  </div>
                </div>
              </div>
            </Card>
          {:else}
            <Card>
              <p class="empty-state">No domains found. Import exams with domain definitions first.</p>
            </Card>
          {/each}
        </div>

        <Card title="Generation Gate Levels">
          <div class="gate-table">
            <div class="gate-row header">
              <span>Level</span><span>Unlocked</span><span>Cost</span>
            </div>
            <div class="gate-row"><span>3+</span><span>Sprint (3Q)</span><span>50 coins (first FREE)</span></div>
            <div class="gate-row"><span>5+</span><span>Custom Sprint</span><span>75 coins</span></div>
            <div class="gate-row"><span>8+</span><span>Full Exam (9Q)</span><span>200 coins</span></div>
            <div class="gate-row"><span>10+</span><span>Cross-domain</span><span>100 coins</span></div>
          </div>
        </Card>
      {/if}
    </div>
  {/if}
</div>

<style>
  .store-page {
    max-width: 1000px;
    margin: 0 auto;
  }

  .store-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: var(--spacing-lg);
    flex-wrap: wrap;
    gap: var(--spacing-md);
  }

  .header-left {
    display: flex;
    align-items: center;
    gap: var(--spacing-lg);
  }

  .page-title {
    font-size: 24px;
    font-weight: 700;
    margin: 0;
  }

  .view-toggle {
    display: flex;
    background: var(--bg-card);
    border-radius: var(--radius-lg);
    padding: 4px;
  }

  .view-toggle button {
    padding: var(--spacing-sm) var(--spacing-md);
    background: transparent;
    border: none;
    border-radius: var(--radius-md);
    color: var(--text-secondary);
    cursor: pointer;
    transition: all 0.2s;
    font-size: 14px;
  }

  .view-toggle button:hover {
    color: var(--text-primary);
  }

  .view-toggle button.active {
    background: var(--primary-600);
    color: white;
  }

  .header-stats {
    display: flex;
    gap: var(--spacing-md);
  }

  .wallet-display, .token-display {
    font-size: 16px;
    padding: var(--spacing-sm) var(--spacing-md);
    background: var(--bg-card);
    border-radius: var(--radius-lg);
    font-weight: 600;
  }

  .coins {
    color: var(--accent-gold);
  }

  .tokens {
    color: var(--primary-400);
  }

  .notification {
    position: fixed;
    top: var(--spacing-lg);
    right: var(--spacing-lg);
    padding: var(--spacing-md) var(--spacing-lg);
    background: var(--primary-600);
    color: white;
    border-radius: var(--radius-md);
    z-index: 1000;
    animation: slideIn 0.3s ease;
  }

  @keyframes slideIn {
    from { transform: translateX(100%); opacity: 0; }
    to { transform: translateX(0); opacity: 1; }
  }

  /* Token overview */
  .token-overview {
    text-align: center;
    padding: var(--spacing-md);
  }

  .token-big {
    display: flex;
    flex-direction: column;
    align-items: center;
    margin-bottom: var(--spacing-sm);
  }

  .token-count {
    font-size: 48px;
    font-weight: 700;
    color: var(--primary-400);
  }

  .token-label {
    color: var(--text-muted);
    font-size: 14px;
  }

  .token-lifetime {
    font-size: 12px;
    color: var(--text-muted);
  }

  /* Section headers */
  .section-title {
    font-size: 18px;
    font-weight: 600;
    margin: var(--spacing-lg) 0 var(--spacing-xs);
  }

  .section-desc {
    color: var(--text-muted);
    font-size: 14px;
    margin-bottom: var(--spacing-lg);
  }

  /* Packs grid */
  .packs-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(220px, 1fr));
    gap: var(--spacing-lg);
    margin-bottom: var(--spacing-xl);
  }

  .pack-card {
    background: var(--bg-card);
    border-radius: var(--radius-lg);
    padding: var(--spacing-xl) var(--spacing-lg);
    text-align: center;
    position: relative;
    transition: transform 0.2s, box-shadow 0.2s;
    border: 2px solid transparent;
  }

  .pack-card:hover {
    transform: translateY(-4px);
    box-shadow: 0 8px 24px rgba(0,0,0,0.2);
    border-color: var(--primary-600);
  }

  .pack-badge {
    position: absolute;
    top: -8px;
    right: 12px;
    background: var(--accent-green);
    color: white;
    padding: 2px 10px;
    border-radius: var(--radius-sm);
    font-size: 11px;
    font-weight: 600;
  }

  .pack-icon {
    font-size: 40px;
    margin-bottom: var(--spacing-sm);
  }

  .pack-name {
    font-size: 16px;
    font-weight: 600;
    margin: 0 0 var(--spacing-xs);
  }

  .pack-amount {
    font-size: 24px;
    font-weight: 700;
    color: var(--primary-400);
    margin-bottom: var(--spacing-xs);
  }

  .pack-per {
    font-size: 12px;
    color: var(--text-muted);
    margin-bottom: var(--spacing-lg);
  }

  .pack-footer {
    display: flex;
    justify-content: center;
    align-items: center;
    gap: var(--spacing-md);
  }

  .pack-price {
    font-weight: 600;
    color: var(--accent-gold);
  }

  /* How it works */
  .how-it-works {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-md);
  }

  .step {
    display: flex;
    align-items: center;
    gap: var(--spacing-md);
    font-size: 14px;
  }

  .step-num {
    width: 28px;
    height: 28px;
    display: flex;
    align-items: center;
    justify-content: center;
    background: var(--primary-600);
    color: white;
    border-radius: 50%;
    font-weight: 600;
    font-size: 13px;
    flex-shrink: 0;
  }

  /* Forge section */
  .forge-status {
    margin-bottom: var(--spacing-sm);
  }

  .ollama-status {
    display: flex;
    align-items: center;
    gap: var(--spacing-sm);
    font-weight: 600;
    margin-bottom: var(--spacing-sm);
  }

  .status-dot {
    width: 10px;
    height: 10px;
    border-radius: 50%;
    background: var(--accent-red);
  }

  .ollama-status.available .status-dot {
    background: var(--accent-green);
  }

  .forge-info {
    color: var(--text-muted);
    font-size: 14px;
  }

  .model-info {
    font-size: 12px;
    color: var(--text-muted);
    margin-top: var(--spacing-xs);
  }

  /* Domain cards */
  .domain-cards {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-md);
    margin-bottom: var(--spacing-lg);
  }

  .domain-card {
    padding: var(--spacing-sm);
  }

  .domain-header {
    display: flex;
    align-items: center;
    gap: var(--spacing-md);
    margin-bottom: var(--spacing-md);
  }

  .domain-icon {
    font-size: 28px;
  }

  .domain-name {
    font-size: 16px;
    font-weight: 600;
    margin: 0;
  }

  .domain-level {
    font-size: 12px;
    color: var(--primary-400);
    font-weight: 600;
  }

  .generation-types {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-sm);
  }

  .gen-type {
    padding: var(--spacing-md);
    background: var(--bg-tertiary);
    border-radius: var(--radius-md);
    transition: opacity 0.2s;
  }

  .gen-type.locked {
    opacity: 0.5;
  }

  .gen-type-header {
    display: flex;
    align-items: center;
    gap: var(--spacing-sm);
    margin-bottom: var(--spacing-sm);
  }

  .gen-lock {
    font-size: 14px;
  }

  .gen-name {
    flex: 1;
    font-weight: 500;
  }

  .gen-unlock {
    font-size: 11px;
    padding: 2px 6px;
    background: var(--bg-card);
    border-radius: var(--radius-sm);
    color: var(--text-muted);
  }

  .gen-action {
    display: flex;
    align-items: center;
    gap: var(--spacing-sm);
  }

  .topic-input {
    flex: 1;
    padding: var(--spacing-xs) var(--spacing-sm);
    background: var(--bg-card);
    border: 1px solid var(--bg-hover);
    border-radius: var(--radius-sm);
    color: var(--text-primary);
    font-size: 13px;
  }

  .free-badge {
    display: inline-block;
    margin-top: var(--spacing-xs);
    font-size: 12px;
    color: var(--accent-green);
    font-weight: 600;
  }

  /* Generating overlay */
  .generating-overlay {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: var(--spacing-md);
    padding: var(--spacing-xl);
    text-align: center;
    color: var(--text-primary);
  }

  .generating-sub {
    font-size: 12px;
    color: var(--text-muted);
  }

  .spinner {
    width: 32px;
    height: 32px;
    border: 3px solid var(--bg-tertiary);
    border-top-color: var(--primary-500);
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }

  /* Gate table */
  .gate-table {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-xs);
  }

  .gate-row {
    display: grid;
    grid-template-columns: 60px 1fr 1fr;
    gap: var(--spacing-md);
    padding: var(--spacing-sm);
    font-size: 13px;
    border-radius: var(--radius-sm);
  }

  .gate-row.header {
    font-weight: 600;
    color: var(--text-muted);
    border-bottom: 1px solid var(--bg-tertiary);
  }

  .gate-row:not(.header) {
    background: var(--bg-tertiary);
  }

  .empty-state {
    text-align: center;
    color: var(--text-muted);
    padding: var(--spacing-xl);
    grid-column: 1 / -1;
  }
</style>

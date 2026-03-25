<script>
  import { onMount, onDestroy } from 'svelte';
  import Card from '../lib/components/Card.svelte';
  import Button from '../lib/components/Button.svelte';

  let notification = '';

  function showNotification(msg) {
    notification = msg;
    setTimeout(() => notification = '', 3000);
  }

  let settings = {
    theme: 'dark',
    voiceEnabled: true,
    voiceSpeed: 1.0,
    passThreshold: 60,
    showTimer: true,
    revealAttempts: 2,
    desktopAlerts: true,
    streakReminder: true,
    reminderTime: '20:00',
    achievementPopup: true,
    soundEffects: true,
  };

  // ── Ollama state ──
  let ollamaConfig = { base_url: 'http://localhost:11434', model: 'llama3.1:8b', timeout_seconds: 120, max_retries: 2 };
  let systemInfo = null;
  let ollamaAvailable = false;
  let ollamaModels = [];
  let showAdvanced = false;

  // Pull state
  let pullProgress = null;
  let pullInterval = null;

  // Test state
  let testResults = null;
  let testing = false;

  // Model input
  let customModel = '';
  let showCustomModel = false;

  onMount(async () => {
    await loadOllamaConfig();
    await loadSystemInfo();
    await checkOllama();
  });

  onDestroy(() => {
    if (pullInterval) clearTimeout(pullInterval);
  });

  async function loadOllamaConfig() {
    if (window.go?.main?.App?.GetOllamaConfig) {
      try {
        ollamaConfig = await window.go.main.App.GetOllamaConfig();
      } catch (e) { /* use defaults */ }
    }
  }

  async function loadSystemInfo() {
    if (window.go?.main?.App?.GetSystemInfo) {
      try {
        systemInfo = await window.go.main.App.GetSystemInfo();
      } catch (e) { /* ignore */ }
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

  async function saveOllamaConfig() {
    if (window.go?.main?.App?.UpdateOllamaConfig) {
      try {
        await window.go.main.App.UpdateOllamaConfig(
          ollamaConfig.base_url,
          ollamaConfig.model,
          ollamaConfig.timeout_seconds,
          ollamaConfig.max_retries
        );
        showNotification('Ollama settings saved');
        await checkOllama();
      } catch (err) {
        showNotification('Failed to save: ' + err);
      }
    }
  }

  async function pollPullProgress(model) {
    if (window.go?.main?.App?.GetPullProgress) {
      pullProgress = await window.go.main.App.GetPullProgress();
      if (pullProgress && !pullProgress.active) {
        pullInterval = null;
        if (pullProgress.status === 'complete') {
          showNotification('Model ' + model + ' pulled successfully');
          await checkOllama();
          ollamaConfig.model = model;
        } else if (pullProgress.error) {
          showNotification('Pull failed: ' + pullProgress.error);
        }
        return;
      }
    }
    pullInterval = setTimeout(() => pollPullProgress(model), 500);
  }

  async function pullModel(model) {
    if (window.go?.main?.App?.PullOllamaModel) {
      try {
        await window.go.main.App.PullOllamaModel(model);
        pullInterval = setTimeout(() => pollPullProgress(model), 500);
      } catch (err) {
        showNotification('Pull failed: ' + err);
      }
    }
  }

  async function testConnection() {
    testing = true;
    testResults = null;
    if (window.go?.main?.App?.TestOllamaConnection) {
      try {
        testResults = await window.go.main.App.TestOllamaConnection();
      } catch (err) {
        testResults = { reachable: false, error: '' + err };
      }
    }
    testing = false;
  }

  function gpuIcon(type) {
    switch (type) {
      case 'nvidia': return '🟢';
      case 'amd_rocm': return '🔴';
      case 'amd_no_rocm': return '🟡';
      case 'intel_arc': return '🔵';
      default: return '⚪';
    }
  }

  function gpuLabel(type) {
    switch (type) {
      case 'nvidia': return 'NVIDIA (CUDA)';
      case 'amd_rocm': return 'AMD (ROCm)';
      case 'amd_no_rocm': return 'AMD (no ROCm — CPU mode)';
      case 'intel_arc': return 'Intel Arc (experimental)';
      default: return 'CPU only';
    }
  }

  $: needsPull = systemInfo && ollamaModels.length > 0 &&
    !ollamaModels.some(m => m.includes(systemInfo.rec_model) || systemInfo.rec_model.includes(m));

  $: isPulling = pullProgress && pullProgress.active;

  function saveSettings() {
    // TODO: Save to backend
    console.log('Saving settings:', settings);
  }

  function exportData() {
    // TODO: Implement export
    console.log('Exporting data...');
  }

  function importData() {
    // TODO: Implement import
    console.log('Importing data...');
  }

  function resetProgress() {
    if (confirm('Are you sure? This will reset all your progress!')) {
      console.log('Resetting progress...');
    }
  }
</script>

<div class="settings-page">
  <h1 class="page-title">Settings</h1>

  {#if notification}
    <div class="notification">{notification}</div>
  {/if}

  <div class="settings-content">
    <Card title="General">
      <div class="setting-row">
        <label>Theme</label>
        <select bind:value={settings.theme}>
          <option value="dark">Dark (Cozy Purple)</option>
          <option value="light">Light</option>
        </select>
      </div>
    </Card>

    <Card title="Voice">
      <div class="setting-row">
        <label>Enable Voice Mode</label>
        <input type="checkbox" bind:checked={settings.voiceEnabled} />
      </div>
      <div class="setting-row">
        <label>Speech Rate</label>
        <input
          type="range"
          min="0.5"
          max="2"
          step="0.1"
          bind:value={settings.voiceSpeed}
        />
        <span>{settings.voiceSpeed}x</span>
      </div>
    </Card>

    <!-- LLM / Ollama Settings -->
    <Card title="LLM / Ollama">
      <!-- Hardware Info Card -->
      {#if systemInfo}
        <div class="hw-info-card">
          <div class="hw-row">
            <span class="hw-label">GPU</span>
            <span class="hw-value">
              {gpuIcon(systemInfo.gpu_type)} {systemInfo.gpu_name}
              {#if systemInfo.vram_gb > 0}
                ({systemInfo.vram_gb}GB VRAM)
              {/if}
            </span>
          </div>
          <div class="hw-row">
            <span class="hw-label">RAM</span>
            <span class="hw-value">{systemInfo.ram_gb}GB</span>
          </div>
          <div class="hw-row">
            <span class="hw-label">Accel</span>
            <span class="hw-value">{gpuLabel(systemInfo.gpu_type)}</span>
          </div>
          <div class="hw-rec">
            Recommended: <strong>{systemInfo.rec_model}</strong> ({systemInfo.rec_size})
            <span class="hw-reason">{systemInfo.rec_reason}</span>
          </div>
          {#if systemInfo.gpu_type === 'amd_no_rocm'}
            <div class="hw-tip">
              Enable ROCm in NixOS for GPU acceleration:
              <code>hardware.amdgpu.opencl.enable = true;</code>
            </div>
          {/if}
        </div>
      {:else}
        <div class="hw-info-card loading">Detecting hardware...</div>
      {/if}

      <!-- Connection Status -->
      <div class="setting-row">
        <label>Connection</label>
        <div class="ollama-status" class:available={ollamaAvailable}>
          <span class="status-dot"></span>
          <span>{ollamaAvailable ? 'Connected' : 'Offline'}</span>
          {#if ollamaAvailable && ollamaModels.length > 0}
            <span class="model-count">({ollamaModels.length} models)</span>
          {/if}
        </div>
      </div>

      <!-- Base URL -->
      <div class="setting-row">
        <label>Base URL</label>
        <input
          type="text"
          class="text-input"
          bind:value={ollamaConfig.base_url}
          placeholder="http://localhost:11434"
        />
      </div>

      <!-- Model Selection -->
      <div class="setting-row">
        <label>Model</label>
        {#if ollamaModels.length > 0 && !showCustomModel}
          <select bind:value={ollamaConfig.model}>
            {#each ollamaModels as m}
              <option value={m}>{m}</option>
            {/each}
          </select>
          <button class="link-btn" on:click={() => showCustomModel = true}>custom</button>
        {:else}
          <input
            type="text"
            class="text-input"
            bind:value={ollamaConfig.model}
            placeholder="llama3.1:8b"
          />
          {#if ollamaModels.length > 0}
            <button class="link-btn" on:click={() => showCustomModel = false}>list</button>
          {/if}
        {/if}
      </div>

      <!-- Model Pull -->
      {#if isPulling}
        <div class="pull-section">
          <div class="pull-header">
            Pulling <strong>{pullProgress.model}</strong>...
          </div>
          <div class="progress-bar">
            <div class="progress-fill" style="width: {pullProgress.percent.toFixed(0)}%"></div>
          </div>
          <div class="pull-status">
            {pullProgress.status} — {pullProgress.percent.toFixed(1)}%
          </div>
        </div>
      {:else if systemInfo && needsPull && ollamaAvailable}
        <div class="pull-section">
          <div class="pull-suggest">
            <strong>{systemInfo.rec_model}</strong> ({systemInfo.rec_size}) is not installed
          </div>
          <Button variant="secondary" size="small" on:click={() => pullModel(systemInfo.rec_model)}>
            Pull Recommended Model
          </Button>
        </div>
      {/if}

      {#if pullProgress && !pullProgress.active && pullProgress.error}
        <div class="pull-error">{pullProgress.error}</div>
      {/if}

      <!-- Advanced -->
      <details bind:open={showAdvanced}>
        <summary class="advanced-toggle">Advanced</summary>
        <div class="setting-row">
          <label>Timeout (seconds)</label>
          <input
            type="number"
            class="num-input"
            bind:value={ollamaConfig.timeout_seconds}
            min="30"
            max="600"
          />
        </div>
        <div class="setting-row">
          <label>Max Retries</label>
          <input
            type="number"
            class="num-input"
            bind:value={ollamaConfig.max_retries}
            min="0"
            max="5"
          />
        </div>
      </details>

      <!-- Actions -->
      <div class="llm-actions">
        <Button variant="secondary" size="small" on:click={testConnection} disabled={testing}>
          {testing ? 'Testing...' : 'Test Connection'}
        </Button>
        <Button variant="secondary" size="small" on:click={checkOllama}>
          Refresh
        </Button>
        <Button variant="primary" size="small" on:click={saveOllamaConfig}>
          Save
        </Button>
      </div>

      <!-- Test Results -->
      {#if testResults}
        <div class="test-results">
          <div class="test-step" class:pass={testResults.reachable} class:fail={!testResults.reachable}>
            {testResults.reachable ? '✓' : '✗'} Ollama reachable
          </div>
          {#if testResults.reachable}
            <div class="test-step" class:pass={testResults.model_loaded} class:fail={!testResults.model_loaded}>
              {testResults.model_loaded ? '✓' : '✗'} Model loaded ({ollamaConfig.model})
            </div>
          {/if}
          {#if testResults.model_loaded}
            <div class="test-step" class:pass={testResults.generate_ok} class:fail={!testResults.generate_ok}>
              {testResults.generate_ok ? '✓' : '✗'} Test generation
              {#if testResults.response_time_ms > 0}
                ({(testResults.response_time_ms / 1000).toFixed(1)}s)
              {/if}
            </div>
          {/if}
          {#if testResults.error}
            <div class="test-error">{testResults.error}</div>
          {/if}
        </div>
      {/if}
    </Card>

    <Card title="Exams">
      <div class="setting-row">
        <label>Pass Threshold</label>
        <input
          type="range"
          min="40"
          max="80"
          step="5"
          bind:value={settings.passThreshold}
        />
        <span>{settings.passThreshold}%</span>
      </div>
      <div class="setting-row">
        <label>Show Timer</label>
        <input type="checkbox" bind:checked={settings.showTimer} />
      </div>
      <div class="setting-row">
        <label>Reveal answers after</label>
        <select bind:value={settings.revealAttempts}>
          <option value={1}>1 attempt</option>
          <option value={2}>2 attempts</option>
          <option value={3}>3 attempts</option>
        </select>
      </div>
    </Card>

    <Card title="Notifications">
      <div class="setting-row">
        <label>Desktop Alerts</label>
        <input type="checkbox" bind:checked={settings.desktopAlerts} />
      </div>
      <div class="setting-row">
        <label>Streak Reminder</label>
        <input type="checkbox" bind:checked={settings.streakReminder} />
        <input
          type="time"
          bind:value={settings.reminderTime}
          disabled={!settings.streakReminder}
        />
      </div>
      <div class="setting-row">
        <label>Achievement Popup</label>
        <input type="checkbox" bind:checked={settings.achievementPopup} />
      </div>
      <div class="setting-row">
        <label>Sound Effects</label>
        <input type="checkbox" bind:checked={settings.soundEffects} />
      </div>
    </Card>

    <Card title="Data">
      <div class="data-actions">
        <Button variant="secondary" on:click={exportData}>Export All Data</Button>
        <Button variant="secondary" on:click={importData}>Import Backup</Button>
        <Button variant="danger" on:click={resetProgress}>Reset Progress</Button>
      </div>
      <div class="data-info">
        <p>Database: ~/.local/share/kgate/kgate.db</p>
        <p>Config: ~/.config/kgate/config.toml</p>
      </div>
    </Card>

    <div class="save-actions">
      <Button on:click={saveSettings}>Save Settings</Button>
    </div>
  </div>
</div>

<style>
  .settings-page {
    max-width: 700px;
    margin: 0 auto;
  }

  .page-title {
    font-size: 24px;
    font-weight: 700;
    margin-bottom: var(--spacing-lg);
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

  .settings-content {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-lg);
  }

  .setting-row {
    display: flex;
    align-items: center;
    gap: var(--spacing-md);
    padding: var(--spacing-sm) 0;
    border-bottom: 1px solid var(--bg-tertiary);
  }

  .setting-row:last-child {
    border-bottom: none;
  }

  .setting-row label {
    flex: 1;
  }

  .setting-row select,
  .setting-row input[type="time"] {
    padding: var(--spacing-xs) var(--spacing-sm);
    background: var(--bg-tertiary);
    border: 1px solid var(--bg-hover);
    border-radius: var(--radius-sm);
    color: var(--text-primary);
    font-size: 14px;
  }

  .setting-row input[type="checkbox"] {
    width: 20px;
    height: 20px;
    accent-color: var(--primary-500);
  }

  .setting-row input[type="range"] {
    width: 120px;
    accent-color: var(--primary-500);
  }

  .setting-row span {
    min-width: 40px;
    text-align: right;
    color: var(--text-muted);
    font-size: 13px;
  }

  /* Text & number inputs */
  .text-input {
    padding: var(--spacing-xs) var(--spacing-sm);
    background: var(--bg-tertiary);
    border: 1px solid var(--bg-hover);
    border-radius: var(--radius-sm);
    color: var(--text-primary);
    font-size: 13px;
    font-family: monospace;
    min-width: 180px;
  }

  .num-input {
    padding: var(--spacing-xs) var(--spacing-sm);
    background: var(--bg-tertiary);
    border: 1px solid var(--bg-hover);
    border-radius: var(--radius-sm);
    color: var(--text-primary);
    font-size: 14px;
    width: 80px;
  }

  .link-btn {
    background: none;
    border: none;
    color: var(--primary-400);
    font-size: 12px;
    cursor: pointer;
    text-decoration: underline;
    padding: 0;
  }

  .link-btn:hover {
    color: var(--primary-300);
  }

  /* Hardware info card */
  .hw-info-card {
    background: var(--bg-tertiary);
    border-radius: var(--radius-md);
    padding: var(--spacing-md);
    margin-bottom: var(--spacing-md);
    font-size: 13px;
  }

  .hw-info-card.loading {
    color: var(--text-muted);
    text-align: center;
    padding: var(--spacing-lg);
  }

  .hw-row {
    display: flex;
    gap: var(--spacing-md);
    padding: 3px 0;
  }

  .hw-label {
    color: var(--text-muted);
    min-width: 40px;
    font-weight: 600;
    font-size: 11px;
    text-transform: uppercase;
  }

  .hw-value {
    color: var(--text-primary);
  }

  .hw-rec {
    margin-top: var(--spacing-sm);
    padding-top: var(--spacing-sm);
    border-top: 1px solid var(--bg-hover);
    color: var(--text-secondary);
  }

  .hw-rec strong {
    color: var(--primary-300);
  }

  .hw-reason {
    display: block;
    font-size: 12px;
    color: var(--text-muted);
    margin-top: 2px;
  }

  .hw-tip {
    margin-top: var(--spacing-sm);
    padding: var(--spacing-sm);
    background: rgba(250, 204, 21, 0.1);
    border-radius: var(--radius-sm);
    color: var(--accent-gold);
    font-size: 12px;
  }

  .hw-tip code {
    display: block;
    margin-top: 4px;
    font-size: 11px;
    color: var(--text-primary);
  }

  /* Ollama status */
  .ollama-status {
    display: flex;
    align-items: center;
    gap: var(--spacing-xs);
    font-size: 13px;
  }

  .status-dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: var(--accent-red);
  }

  .ollama-status.available .status-dot {
    background: var(--accent-green);
  }

  .model-count {
    color: var(--text-muted);
    font-size: 12px;
  }

  /* Pull progress */
  .pull-section {
    padding: var(--spacing-md) 0;
  }

  .pull-header, .pull-suggest {
    font-size: 13px;
    color: var(--text-secondary);
    margin-bottom: var(--spacing-sm);
  }

  .progress-bar {
    height: 6px;
    background: var(--bg-tertiary);
    border-radius: 3px;
    overflow: hidden;
    margin-bottom: var(--spacing-xs);
  }

  .progress-fill {
    height: 100%;
    background: var(--primary-500);
    border-radius: 3px;
    transition: width 0.3s ease;
  }

  .pull-status {
    font-size: 12px;
    color: var(--text-muted);
  }

  .pull-error {
    font-size: 12px;
    color: var(--accent-red);
    padding: var(--spacing-xs) 0;
  }

  /* Advanced toggle */
  .advanced-toggle {
    font-size: 13px;
    color: var(--text-muted);
    cursor: pointer;
    padding: var(--spacing-sm) 0;
    user-select: none;
  }

  .advanced-toggle:hover {
    color: var(--text-secondary);
  }

  /* LLM actions */
  .llm-actions {
    display: flex;
    gap: var(--spacing-sm);
    padding-top: var(--spacing-md);
    border-top: 1px solid var(--bg-tertiary);
    margin-top: var(--spacing-sm);
  }

  /* Test results */
  .test-results {
    margin-top: var(--spacing-md);
    padding: var(--spacing-md);
    background: var(--bg-tertiary);
    border-radius: var(--radius-md);
  }

  .test-step {
    font-size: 13px;
    padding: 3px 0;
  }

  .test-step.pass {
    color: var(--accent-green);
  }

  .test-step.fail {
    color: var(--accent-red);
  }

  .test-error {
    font-size: 12px;
    color: var(--accent-red);
    margin-top: var(--spacing-xs);
    padding-top: var(--spacing-xs);
    border-top: 1px solid var(--bg-hover);
  }

  .data-actions {
    display: flex;
    gap: var(--spacing-md);
    margin-bottom: var(--spacing-lg);
    flex-wrap: wrap;
  }

  .data-info {
    font-size: 12px;
    color: var(--text-muted);
  }

  .data-info p {
    margin: var(--spacing-xs) 0;
  }

  .save-actions {
    display: flex;
    justify-content: flex-end;
  }
</style>

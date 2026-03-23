<script>
  import Card from '../lib/components/Card.svelte';
  import Button from '../lib/components/Button.svelte';

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

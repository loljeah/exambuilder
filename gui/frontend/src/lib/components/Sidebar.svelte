<script>
  import { createEventDispatcher } from 'svelte';
  import { dashboard } from '../stores/dashboard.js';
  import Avatar from './Avatar.svelte';

  export let currentPage = 'dashboard';

  const dispatch = createEventDispatcher();

  const navItems = [
    { id: 'dashboard', icon: '📊', label: 'Dashboard', shortcut: '⌘1' },
    { id: 'projects', icon: '📁', label: 'Projects', shortcut: '⌘2' },
    { id: 'exams', icon: '📝', label: 'Take Exam', shortcut: '⌘3' },
    { id: 'knowledge', icon: '📚', label: 'Knowledge', shortcut: '⌘4' },
    { id: 'review', icon: '🔄', label: 'Review', shortcut: '⌘5' },
    { id: 'stats', icon: '📈', label: 'Statistics', shortcut: '⌘6' },
    { id: 'shop', icon: '🛒', label: 'Shop', shortcut: '⌘7' },
    { id: 'achievements', icon: '🏆', label: 'Achievements', shortcut: '⌘8' },
    { id: 'settings', icon: '⚙️', label: 'Settings', shortcut: '⌘,' },
  ];

  function navigate(page) {
    console.log('Sidebar navigate clicked:', page);
    dispatch('navigate', page);
  }

  $: avatarData = $dashboard.avatar || { creature_type: 'cat', name: 'Companion', mood: 'neutral' };
  $: walletData = $dashboard.wallet || { coins: 0 };
  $: profileData = $dashboard.profile || { current_streak: 0 };
</script>

<aside class="sidebar">
  <div class="sidebar-header">
    <div class="avatar-preview">
      <Avatar
        creature={avatarData.creature_type}
        mood={avatarData.mood}
        size="medium"
      />
    </div>
    <div class="avatar-info">
      <span class="avatar-name">{avatarData.name}</span>
      <div class="quick-stats">
        <span class="stat coins">💰 {walletData.coins}</span>
        <span class="stat streak">🔥 {profileData.current_streak}</span>
      </div>
    </div>
  </div>

  <nav class="sidebar-nav">
    {#each navItems as item}
      <button
        class="nav-item"
        class:active={currentPage === item.id}
        on:click={() => navigate(item.id)}
      >
        <span class="nav-icon">{item.icon}</span>
        <span class="nav-label">{item.label}</span>
        <span class="nav-shortcut">{item.shortcut}</span>
      </button>
    {/each}
  </nav>

  <div class="sidebar-footer">
    <div class="status">
      <span class="status-dot"></span>
      <span>Connected</span>
    </div>
    <span class="version">v1.0.0</span>
  </div>
</aside>

<style>
  .sidebar {
    width: 240px;
    background: var(--bg-secondary);
    display: flex;
    flex-direction: column;
    border-right: 1px solid var(--bg-tertiary);
  }

  .sidebar-header {
    padding: var(--spacing-lg);
    border-bottom: 1px solid var(--bg-tertiary);
    display: flex;
    gap: var(--spacing-md);
    align-items: center;
  }

  .avatar-preview {
    flex-shrink: 0;
  }

  .avatar-info {
    flex: 1;
    min-width: 0;
  }

  .avatar-name {
    display: block;
    font-weight: 600;
    color: var(--text-primary);
    margin-bottom: var(--spacing-xs);
  }

  .quick-stats {
    display: flex;
    gap: var(--spacing-sm);
    font-size: 12px;
  }

  .stat {
    color: var(--text-secondary);
  }

  .stat.coins {
    color: var(--accent-gold);
  }

  .stat.streak {
    color: var(--accent-red);
  }

  .sidebar-nav {
    flex: 1;
    padding: var(--spacing-md);
    display: flex;
    flex-direction: column;
    gap: var(--spacing-xs);
  }

  .nav-item {
    display: flex;
    align-items: center;
    gap: var(--spacing-sm);
    padding: var(--spacing-sm) var(--spacing-md);
    border: none;
    background: transparent;
    color: var(--text-secondary);
    cursor: pointer;
    border-radius: var(--radius-md);
    transition: all 0.15s ease;
    text-align: left;
    font-size: 14px;
  }

  .nav-item:hover {
    background: var(--bg-hover);
    color: var(--text-primary);
  }

  .nav-item.active {
    background: var(--primary-600);
    color: white;
  }

  .nav-icon {
    font-size: 16px;
    width: 24px;
    text-align: center;
  }

  .nav-label {
    flex: 1;
  }

  .nav-shortcut {
    font-size: 11px;
    color: var(--text-muted);
    opacity: 0.6;
  }

  .nav-item.active .nav-shortcut {
    color: rgba(255,255,255,0.6);
  }

  .sidebar-footer {
    padding: var(--spacing-md);
    border-top: 1px solid var(--bg-tertiary);
    display: flex;
    justify-content: space-between;
    align-items: center;
    font-size: 12px;
    color: var(--text-muted);
  }

  .status {
    display: flex;
    align-items: center;
    gap: var(--spacing-xs);
  }

  .status-dot {
    width: 8px;
    height: 8px;
    background: var(--accent-green);
    border-radius: 50%;
  }

  .version {
    opacity: 0.6;
  }
</style>

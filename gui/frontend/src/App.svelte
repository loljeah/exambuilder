<script>
  import { onMount } from 'svelte';
  import Sidebar from './lib/components/Sidebar.svelte';
  import Dashboard from './pages/Dashboard.svelte';
  import Exams from './pages/Exams.svelte';
  import KnowledgeBase from './pages/KnowledgeBase.svelte';
  import Review from './pages/Review.svelte';
  import Stats from './pages/Stats.svelte';
  import Shop from './pages/Shop.svelte';
  import Achievements from './pages/Achievements.svelte';
  import Settings from './pages/Settings.svelte';
  import { dashboard } from './lib/stores/dashboard.js';

  let currentPage = 'dashboard';

  const pages = {
    dashboard: Dashboard,
    exams: Exams,
    knowledge: KnowledgeBase,
    review: Review,
    stats: Stats,
    shop: Shop,
    achievements: Achievements,
    settings: Settings,
  };

  function navigate(page) {
    currentPage = page;
  }

  // Keyboard shortcuts
  function handleKeydown(e) {
    if (e.ctrlKey || e.metaKey) {
      switch(e.key) {
        case '1': navigate('dashboard'); e.preventDefault(); break;
        case '2': navigate('exams'); e.preventDefault(); break;
        case '3': navigate('knowledge'); e.preventDefault(); break;
        case '4': navigate('review'); e.preventDefault(); break;
        case '5': navigate('stats'); e.preventDefault(); break;
        case '6': navigate('shop'); e.preventDefault(); break;
        case '7': navigate('achievements'); e.preventDefault(); break;
        case ',': navigate('settings'); e.preventDefault(); break;
      }
    }
  }

  onMount(() => {
    try {
      dashboard.refresh();
    } catch (err) {
      console.error('Dashboard refresh error:', err);
    }
  });
</script>

<svelte:window on:keydown={handleKeydown} />

<div class="app-container">
  <Sidebar {currentPage} on:navigate={(e) => navigate(e.detail)} />

  <main class="main-content">
    <svelte:component this={pages[currentPage]} />
  </main>
</div>

<style>
  .app-container {
    display: flex;
    height: 100vh;
    overflow: hidden;
  }

  .main-content {
    flex: 1;
    overflow-y: auto;
    padding: var(--spacing-lg);
    background: var(--bg-primary);
  }
</style>

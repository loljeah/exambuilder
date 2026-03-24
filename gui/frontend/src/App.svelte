<script>
  import { onMount } from 'svelte';
  import Sidebar from './lib/components/Sidebar.svelte';
  import Dashboard from './pages/Dashboard.svelte';
  import Projects from './pages/Projects.svelte';
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
    projects: Projects,
    exams: Exams,
    knowledge: KnowledgeBase,
    review: Review,
    stats: Stats,
    shop: Shop,
    achievements: Achievements,
    settings: Settings,
  };

  function navigate(page) {
    console.log('App navigate:', page);
    currentPage = page;
  }

  // Keyboard shortcuts
  function handleKeydown(e) {
    if (e.ctrlKey || e.metaKey) {
      switch(e.key) {
        case '1': navigate('dashboard'); e.preventDefault(); break;
        case '2': navigate('projects'); e.preventDefault(); break;
        case '3': navigate('exams'); e.preventDefault(); break;
        case '4': navigate('knowledge'); e.preventDefault(); break;
        case '5': navigate('review'); e.preventDefault(); break;
        case '6': navigate('stats'); e.preventDefault(); break;
        case '7': navigate('shop'); e.preventDefault(); break;
        case '8': navigate('achievements'); e.preventDefault(); break;
        case ',': navigate('settings'); e.preventDefault(); break;
      }
    }
  }

  onMount(() => {
    // Global error handlers to prevent webkit freezing
    window.onerror = (message, source, lineno, colno, error) => {
      console.error('Global error:', message, source, lineno, colno, error);
      return true; // Prevent default handling
    };

    window.onunhandledrejection = (event) => {
      console.error('Unhandled promise rejection:', event.reason);
      event.preventDefault();
    };

    // Defer dashboard load to not block initial render
    setTimeout(() => {
      dashboard.refresh().catch(err => {
        console.error('Dashboard refresh failed:', err);
      });
    }, 100);
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

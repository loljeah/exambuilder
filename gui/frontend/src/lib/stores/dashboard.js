import { writable } from 'svelte/store';

// Helper to add timeout to promises
function withTimeout(promise, ms = 5000) {
  return Promise.race([
    promise,
    new Promise((_, reject) =>
      setTimeout(() => reject(new Error('Request timeout')), ms)
    )
  ]);
}

function createDashboardStore() {
  const { subscribe, set, update } = writable({
    profile: { level: 1, total_xp: 0, current_streak: 0, best_streak: 0, sprints_passed: 0 },
    avatar: { creature_type: 'cat', name: 'Whiskers', mood: 'neutral', xp_multiplier: 1.0 },
    wallet: { coins: 0, lifetime_coins: 0 },
    daily_login: { current_day: 0, total_claims: 0, can_claim: false },
    challenges: [],
    weekly_goals: [],
    review_due: 0,
    active_project: null,
    pending_sprints: 0,
    loading: true,
    error: null,
  });

  return {
    subscribe,
    refresh: async () => {
      try {
        update(d => ({ ...d, loading: true, error: null }));
        // Call Wails backend
        if (window.go?.main?.App?.GetDashboardData) {
          console.log('Dashboard: calling GetDashboardData');
          const data = await withTimeout(window.go.main.App.GetDashboardData(), 5000);
          console.log('Dashboard: got data', data ? 'success' : 'empty');
          update(d => ({ ...d, ...data, loading: false }));
        } else {
          // Demo data for development
          console.log('Dashboard: GetDashboardData not available');
          update(d => ({ ...d, loading: false }));
        }
      } catch (err) {
        console.error('Dashboard refresh error:', err);
        update(d => ({ ...d, loading: false, error: err?.message || 'Unknown error' }));
      }
    },
    setActiveProject: async (projectId) => {
      try {
        if (window.go?.main?.App?.SetActiveProject) {
          console.log('Dashboard: setting active project', projectId);
          await withTimeout(window.go.main.App.SetActiveProject(projectId), 5000);
        }
      } catch (err) {
        console.error('Dashboard setActiveProject error:', err);
      }
    }
  };
}

export const dashboard = createDashboardStore();

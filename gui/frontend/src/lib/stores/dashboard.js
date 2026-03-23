import { writable } from 'svelte/store';

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
          const data = await window.go.main.App.GetDashboardData();
          update(d => ({ ...d, ...data, loading: false }));
        } else {
          // Demo data for development
          update(d => ({ ...d, loading: false }));
        }
      } catch (err) {
        update(d => ({ ...d, loading: false, error: err.message }));
      }
    },
    setActiveProject: async (projectId) => {
      if (window.go?.main?.App?.SetActiveProject) {
        await window.go.main.App.SetActiveProject(projectId);
      }
    }
  };
}

export const dashboard = createDashboardStore();

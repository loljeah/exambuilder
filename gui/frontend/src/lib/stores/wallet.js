import { writable } from 'svelte/store';

function createWalletStore() {
  const { subscribe, set, update } = writable({
    coins: 0,
    lifetime_coins: 0,
    loading: false,
  });

  return {
    subscribe,
    refresh: async () => {
      if (window.go?.main?.App?.GetWallet) {
        const data = await window.go.main.App.GetWallet();
        set({ ...data, loading: false });
      }
    },
    spend: (amount) => {
      update(w => ({ ...w, coins: w.coins - amount }));
    },
    earn: (amount) => {
      update(w => ({
        ...w,
        coins: w.coins + amount,
        lifetime_coins: w.lifetime_coins + amount
      }));
    }
  };
}

export const wallet = createWalletStore();

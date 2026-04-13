import { create } from 'zustand';

type Theme = 'light' | 'dark' | 'system';

interface ThemeState {
  theme: Theme;
  setTheme: (theme: Theme) => void;
}

export const useThemeStore = create<ThemeState>()((set) => ({
  theme: (localStorage.getItem('kanva-theme') as Theme) || 'system',
  setTheme: (theme) => {
    localStorage.setItem('kanva-theme', theme);
    set({ theme });
  },
}));

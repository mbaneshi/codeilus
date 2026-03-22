export type Theme = 'light' | 'dark';

function createThemeStore() {
  let theme = $state<Theme>('dark');

  if (typeof window !== 'undefined') {
    const stored = localStorage.getItem('codeilus-theme') as Theme | null;
    theme = stored ?? 'dark';
    applyTheme(theme);
  }

  function applyTheme(t: Theme) {
    const root = document.documentElement;
    root.classList.remove('light', 'dark');
    root.classList.add(t);
  }

  return {
    get current() { return theme; },
    toggle() {
      theme = theme === 'dark' ? 'light' : 'dark';
      applyTheme(theme);
      localStorage.setItem('codeilus-theme', theme);
      // Notify components that need to react to theme changes
      if (typeof window !== 'undefined') {
        window.dispatchEvent(new CustomEvent('theme-change', { detail: theme }));
      }
    }
  };
}

export const themeStore = createThemeStore();

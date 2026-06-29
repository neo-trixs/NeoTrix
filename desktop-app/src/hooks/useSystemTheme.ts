import { useState, useEffect } from 'react';

export function useSystemTheme(): 'light' | 'dark' {
  const prefersDark = matchMedia?.('(prefers-color-scheme: dark)').matches ?? true;
  const [theme, setTheme] = useState<'light' | 'dark'>(prefersDark ? 'dark' : 'light');

  useEffect(() => {
    const mql = window.matchMedia('(prefers-color-scheme: dark)');
    function onChange(e: MediaQueryListEvent) {
      setTheme(e.matches ? 'dark' : 'light');
    }
    mql.addEventListener('change', onChange);
    return () => mql.removeEventListener('change', onChange);
  }, []);

  return theme;
}

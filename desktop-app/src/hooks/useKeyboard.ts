import { useEffect } from 'react';

interface Shortcut {
  key: string;
  metaKey?: boolean;
  ctrlKey?: boolean;
  shiftKey?: boolean;
  handler: () => void;
}

function match(e: KeyboardEvent, s: Shortcut): boolean {
  const isMeta = e.metaKey || e.ctrlKey;
  if (s.metaKey && !e.metaKey) return false;
  if (s.ctrlKey && !e.ctrlKey) return false;
  if (s.shiftKey && !e.shiftKey) return false;
  if ((s.metaKey || s.ctrlKey) && !isMeta) return false;
  return e.key.toLowerCase() === s.key.toLowerCase();
}

export function useKeyboard(shortcuts: Shortcut[]) {
  useEffect(() => {
    function onKeyDown(e: KeyboardEvent) {
      if (
        e.target instanceof HTMLInputElement ||
        e.target instanceof HTMLTextAreaElement ||
        e.target instanceof HTMLSelectElement
      ) {
        return;
      }
      for (const s of shortcuts) {
        if (match(e, s)) {
          e.preventDefault();
          s.handler();
          return;
        }
      }
    }
    window.addEventListener('keydown', onKeyDown);
    return () => window.removeEventListener('keydown', onKeyDown);
  }, [shortcuts]);
}

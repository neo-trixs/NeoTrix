let loadPromise: Promise<typeof import('@xterm/xterm')> | null = null;

export async function getXterm(): Promise<typeof import('@xterm/xterm')> {
  if (!loadPromise) {
    loadPromise = import('@xterm/xterm');
    await import('@xterm/xterm/css/xterm.css');
  }
  return loadPromise;
}

export async function getXtermAddonFit(): Promise<typeof import('@xterm/addon-fit')> {
  return import('@xterm/addon-fit');
}

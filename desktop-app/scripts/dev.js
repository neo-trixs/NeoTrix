/**
 * Cross-platform dev launcher.
 * Usage: node scripts/dev.js
 * Ensures env vars are set correctly per platform before vite dev.
 */
import { spawn } from 'child_process';
import { resolve, dirname } from 'path';
import { fileURLToPath } from 'url';

const __dirname = dirname(fileURLToPath(import.meta.url));
const root = resolve(__dirname, '..');

const isWindows = process.platform === 'win32';
const cmd = isWindows ? 'npx.cmd' : 'npx';

const child = spawn(cmd, ['vite', '--host'], {
  cwd: root,
  stdio: 'inherit',
  shell: true,
  env: {
    ...process.env,
    BROWSER: 'none',
  },
});

child.on('exit', code => process.exit(code ?? 1));

/**
 * Cross-platform setup script for NeoTrix Desktop.
 * Usage: node scripts/setup.js
 */
import { execSync } from 'child_process';
import { existsSync, mkdirSync, writeFileSync } from 'fs';
import { resolve, dirname } from 'path';
import { fileURLToPath } from 'url';

const __dirname = dirname(fileURLToPath(import.meta.url));
const root = resolve(__dirname, '..');

function run(cmd) {
  console.log(`> ${cmd}`);
  execSync(cmd, { cwd: root, stdio: 'inherit' });
}

function checkSystem() {
  const platform = process.platform;
  console.log(`Platform: ${platform}`);

  if (platform === 'win32') {
    console.log('Windows detected — using npm scripts with cross-env');
  } else if (platform === 'darwin') {
    console.log('macOS detected');
  } else if (platform === 'linux') {
    console.log('Linux detected');
  } else {
    console.warn(`Unknown platform: ${platform}`);
  }
}

function ensureDirs() {
  ['public/icons', 'src/plugins'].forEach(dir => {
    const p = resolve(root, dir);
    if (!existsSync(p)) {
      mkdirSync(p, { recursive: true });
      console.log(`Created: ${dir}/`);
    }
  });
}

console.log('\n=== NeoTrix Desktop Setup ===\n');
checkSystem();
ensureDirs();
run('npm install');
run('npm run build');

console.log('\nSetup complete. Run `npm run dev` to start.\n');

import { test, expect } from '@playwright/test';
import { execSync } from 'child_process';
import { resolve } from 'path';

const BIN = process.env.NEOTRIX_BIN || resolve(__dirname, '..', 'target', 'release', 'neotrix');

function run(args: string): { stdout: string; stderr: string; exitCode: number } {
  try {
    const stdout = execSync(`"${BIN}" ${args}`, {
      encoding: 'utf-8',
      timeout: 30_000,
    });
    return { stdout: stdout.trim(), stderr: '', exitCode: 0 };
  } catch (e: any) {
    return {
      stdout: (e.stdout || '').toString().trim(),
      stderr: (e.stderr || '').toString().trim(),
      exitCode: e.status ?? 1,
    };
  }
}

test.describe('neotrix CLI smoke tests', () => {
  test('neotrix --help exits with 0 and shows help text', () => {
    const { stdout, exitCode } = run('--help');
    expect(exitCode).toBe(0);
    expect(stdout).toContain('Usage');
    expect(stdout).toContain('neotrix');
  });

  test('neotrix --version shows version number', () => {
    const { stdout, exitCode } = run('--version');
    expect(exitCode).toBe(0);
    expect(stdout).toMatch(/\d+\.\d+\.\d+/);
  });

  test('neotrix exec "say hello" runs without error', () => {
    const { exitCode, stderr } = run('exec "say hello"');
    expect(exitCode).toBe(0);
  });

  test('neotrix exec --json "say hello" produces valid JSON output', () => {
    const { stdout, exitCode } = run('exec --json "say hello"');
    expect(exitCode).toBe(0);
    expect(() => JSON.parse(stdout)).not.toThrow();
  });
});

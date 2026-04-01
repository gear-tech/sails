import { readdirSync } from 'node:fs';
import { resolve } from 'node:path';

const testDir = resolve(import.meta.dir);
const testFiles = readdirSync(testDir)
  .filter((file) => file.endsWith('.test.ts'))
  .map((file) => resolve(testDir, file));

const proc = Bun.spawn([process.execPath, 'test', ...testFiles, '--timeout', '15000'], {
  cwd: resolve(import.meta.dir, '..'),
  stdout: 'inherit',
  stderr: 'inherit',
});

const exitCode = await proc.exited;

if (exitCode !== 0) {
  process.exit(exitCode);
}

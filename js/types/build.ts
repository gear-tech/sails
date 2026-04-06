import { rmSync } from 'node:fs';
import { resolve } from 'node:path';

const TSC_BIN = resolve(import.meta.dir, '../../node_modules/typescript/bin/tsc');

async function runTsc(args: string[]) {
  const proc = Bun.spawn([process.execPath, TSC_BIN, ...args], {
    cwd: import.meta.dir,
    stdout: 'inherit',
    stderr: 'inherit',
  });

  const exitCode = await proc.exited;

  if (exitCode !== 0) {
    throw new Error(`tsc exited with code ${exitCode}`);
  }
}

rmSync(resolve(import.meta.dir, 'lib'), { recursive: true, force: true });

await runTsc(['-p', 'tsconfig.json']);


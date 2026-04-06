import { rmSync } from 'node:fs';
import { resolve } from 'node:path';

const TSC_BIN = resolve(import.meta.dir, '../../node_modules/typescript/bin/tsc');

function ensureBuild(result: BuildOutput, label: string) {
  if (result.success) {
    return;
  }

  for (const log of result.logs) {
    console.error(log);
  }

  throw new Error(`${label} build failed`);
}

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

ensureBuild(
  await Bun.build({
    entrypoints: ['src/index.ts'],
    outdir: 'lib',
    format: 'esm',
    target: 'node',
  }),
  'ESM',
);

ensureBuild(
  await Bun.build({
    entrypoints: ['src/index.ts'],
    outdir: 'lib/cjs',
    format: 'cjs',
    target: 'node',
    naming: '[dir]/[name].cjs',
  }),
  'CJS',
);

await runTsc(['-p', 'tsconfig.json', '--emitDeclarationOnly', '--outDir', 'lib']);


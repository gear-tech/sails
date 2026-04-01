import { copyFileSync, rmSync } from 'node:fs';
import { resolve } from 'node:path';

const TSC_BIN = resolve(import.meta.dir, '../node_modules/typescript/bin/tsc');
const EXTERNAL = [
  '@gear-js/api',
  '@polkadot/api',
  '@polkadot/api/types',
  '@polkadot/types',
  '@polkadot/types/create',
  '@polkadot/types/types',
  '@polkadot/util',
];

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
    entrypoints: ['src/index.ts', 'src/parser.ts', 'src/util.ts'],
    outdir: 'lib',
    format: 'esm',
    target: 'node',
    external: EXTERNAL,
  }),
  'ESM',
);

ensureBuild(
  await Bun.build({
    entrypoints: ['src/index.ts', 'src/parser.ts', 'src/util.ts'],
    outdir: 'lib/cjs',
    format: 'cjs',
    target: 'node',
    naming: '[dir]/[name].cjs',
    external: EXTERNAL,
  }),
  'CJS',
);

await runTsc(['-p', 'tsconfig.build.json', '--emitDeclarationOnly', '--outDir', 'lib']);
copyFileSync(resolve(import.meta.dir, 'types/lib/index.d.ts'), resolve(import.meta.dir, 'lib/types.d.ts'));

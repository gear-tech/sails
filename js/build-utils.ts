import { fileURLToPath } from 'node:url';

// import.meta.url is standard TypeScript-known; resolve TSC relative to this file
const TSC_BIN = fileURLToPath(new URL('../node_modules/typescript/bin/tsc', import.meta.url));

export async function buildEsmCjs(entrypoints: string[], external?: string[]): Promise<void> {
  ensureBuild(
    await Bun.build({ entrypoints, outdir: 'lib', format: 'esm', target: 'node', external }),
    'ESM',
  );
  ensureBuild(
    await Bun.build({ entrypoints, outdir: 'lib/cjs', format: 'cjs', target: 'node', naming: '[dir]/[name].cjs', external }),
    'CJS',
  );
}

function ensureBuild(result: BuildOutput, label: string): void {
  if (result.success) {
    return;
  }

  for (const log of result.logs) {
    console.error(log);
  }

  throw new Error(`${label} build failed`);
}

// cwd defaults to process.cwd() — Bun always runs scripts with cwd = package dir
export async function runTsc(args: string[]): Promise<void> {
  const proc = Bun.spawn([process.execPath, TSC_BIN, ...args], {
    cwd: process.cwd(),
    stdout: 'inherit',
    stderr: 'inherit',
  });

  const exitCode = await proc.exited;

  if (exitCode !== 0) {
    throw new Error(`tsc exited with code ${exitCode}`);
  }
}

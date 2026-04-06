import { readFileSync, rmSync, writeFileSync } from 'node:fs';
import { resolve } from 'node:path';
import config from '../config.json' with { type: 'json' };

const TSC_BIN = resolve(import.meta.dir, '../../node_modules/typescript/bin/tsc');
const WASM_BYTES_PATH = resolve(import.meta.dir, 'src/wasm-bytes.ts');

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

async function getBase64Parser() {
  const version = config['sails-rs'];
  const url = `https://github.com/gear-tech/sails/releases/download/rs%2Fv${version}/sails_idl_parser.wasm`;
  const response = await fetch(url);

  if (!response.ok || !response.body) {
    throw new Error(`Failed to fetch parser from ${url}`);
  }

  const reader = response.body.pipeThrough(new CompressionStream('gzip')).getReader();
  const chunks: Buffer[] = [];

  while (true) {
    const { done, value } = await reader.read();

    if (done) {
      break;
    }

    if (value) {
      chunks.push(Buffer.from(value));
    }
  }

  return Buffer.concat(chunks).toString('base64');
}

const originalWasmBytesSource = readFileSync(WASM_BYTES_PATH, 'utf8');

try {
  rmSync(resolve(import.meta.dir, 'lib'), { recursive: true, force: true });

  // Bun bundles local imports, so wasm-bytes must contain the real payload before bundling.
  writeFileSync(WASM_BYTES_PATH, `export default ${JSON.stringify(await getBase64Parser())};\n`);

  ensureBuild(
    await Bun.build({
      entrypoints: ['src/index.ts'],
      outdir: 'lib',
      format: 'esm',
      target: 'node',
      external: ['sails-js-types'],
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
      external: ['sails-js-types'],
    }),
    'CJS',
  );

  await runTsc(['-p', 'tsconfig.build.json', '--emitDeclarationOnly', '--outDir', 'lib']);
} finally {
  writeFileSync(WASM_BYTES_PATH, originalWasmBytesSource);
}


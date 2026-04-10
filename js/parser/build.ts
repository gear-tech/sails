import { readFileSync, rmSync, writeFileSync } from 'node:fs';
import { resolve } from 'node:path';
import config from '../config.json' with { type: 'json' };
import { buildEsmCjs, runTsc } from '../build-utils';

const WASM_BYTES_PATH = resolve(import.meta.dir, 'src/wasm-bytes.ts');

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

  await buildEsmCjs(['src/index.ts'], ['sails-js-types']);

  await runTsc(['-p', 'tsconfig.build.json', '--emitDeclarationOnly', '--outDir', 'lib']);
} finally {
  writeFileSync(WASM_BYTES_PATH, originalWasmBytesSource);
}


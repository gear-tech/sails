import { existsSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { gzipSync } from 'node:zlib';
import { buildEsmCjs, runTsc } from '../build-utils';

const WASM_BYTES_PATH = resolve(import.meta.dir, 'src/wasm-bytes.ts');
const WASM_PATH = resolve(import.meta.dir, '../../target/wasm32-unknown-unknown/release/sails_idl_v2_parser.wasm');

function getBase64Parser() {
  if (!existsSync(WASM_PATH)) {
    throw new Error(
      `WASM not found at ${WASM_PATH}\n` +
        'Build it first:\n' +
        '  cargo build -p sails-idl-parser-wasm --target=wasm32-unknown-unknown --release\n' +
        '  wasm-opt -O4 -o ./target/wasm32-unknown-unknown/release/sails_idl_v2_parser.wasm ./target/wasm32-unknown-unknown/release/sails_idl_parser_wasm.wasm',
    );
  }

  return gzipSync(readFileSync(WASM_PATH)).toString('base64');
}

const originalWasmBytesSource = readFileSync(WASM_BYTES_PATH, 'utf8');

try {
  rmSync(resolve(import.meta.dir, 'lib'), { recursive: true, force: true });

  // Bun bundles local imports, so wasm-bytes must contain the real payload before bundling.
  writeFileSync(WASM_BYTES_PATH, `export default ${JSON.stringify(getBase64Parser())};\n`);

  await buildEsmCjs(['src/index.ts'], ['sails-js-types']);

  await runTsc(['-p', 'tsconfig.build.json', '--emitDeclarationOnly', '--outDir', 'lib']);
} finally {
  writeFileSync(WASM_BYTES_PATH, originalWasmBytesSource);
}


import { readFileSync, writeFileSync, rmSync, mkdirSync, existsSync } from 'node:fs';
import { resolve, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';
import { gzipSync } from 'node:zlib';
import commonjs from '@rollup/plugin-commonjs';
import typescript from 'rollup-plugin-typescript2';

const __dirname = dirname(fileURLToPath(import.meta.url));

function getBase64Parser() {
  const parserPath = resolve(__dirname, '../../target/wasm32-unknown-unknown/release/sails_idl_parser.opt.wasm');
  if (!existsSync(parserPath)) {
    throw new Error(`Build sails-idl-parser-wasm project\n
cargo build -p sails-idl-parser-wasm --target=wasm32-unknown-unknown --release\n
wasm-opt -O4 -o ./target/wasm32-unknown-unknown/release/sails_idl_parser.opt.wasm ./target/wasm32-unknown-unknown/release/sails_idl_parser_wasm.wasm\n`);
  }
  const parserBytes = readFileSync(parserPath);
  const compressedBytes = gzipSync(parserBytes);
  return compressedBytes.toString('base64');
}

function writeCompressedWasmParser(type) {
  return {
    name: 'write-wasm-parser',
    async closeBundle() {
      const base64Bytes = getBase64Parser();
      mkdirSync('./lib', { recursive: true });

      if (type === 'cjs') {
        mkdirSync('./lib/cjs', { recursive: true });
        writeFileSync(
          './lib/cjs/wasm-bytes.cjs',
          `Object.defineProperty(exports, '__esModule', { value: true });\n\nvar wasmParserBytes = '${base64Bytes}';\n\nexports.default = wasmParserBytes;`,
        );
      } else {
        writeFileSync('./lib/wasm-bytes.js', `export default '${base64Bytes}'`);
      }
    },
  };
}

function cleanOldBuild() {
  return {
    name: 'clean-old-build',
    buildStart() {
      rmSync('./lib', { recursive: true, force: true });
    },
  };
}

export default [
  {
    input: 'src/index.ts',
    output: [
      {
        dir: 'lib',
        format: 'es',
        preserveModules: true,
        strict: false,
      },
    ],
    plugins: [
      cleanOldBuild(),
      typescript({
        tsconfig: 'tsconfig.build.json',
      }),
      writeCompressedWasmParser('es'),
    ],
  },
  {
    input: 'src/index.ts',
    output: [
      {
        dir: 'lib/cjs',
        format: 'cjs',
        entryFileNames: '[name].cjs',
        preserveModules: true,
        exports: 'named',
        strict: false,
      },
    ],
    plugins: [
      typescript({
        tsconfig: 'tsconfig.cjs.json',
      }),
      commonjs(),
      writeCompressedWasmParser('cjs'),
    ],
  },
];

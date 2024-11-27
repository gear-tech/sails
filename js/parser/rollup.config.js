import { writeFileSync, rmSync } from 'fs';
import commonjs from '@rollup/plugin-commonjs';
import typescript from 'rollup-plugin-typescript2';
import config from '../config.json' assert { type: 'json' };

async function getStreamFromRelease(version, cs) {
  const link = `https://github.com/gear-tech/sails/releases/download/rs%2Fv${version}/sails_idl_parser.wasm`;
  const res = await fetch(link);

  if (!res.ok) {
    throw new Error(`Failed to fetch parser from ${link}`);
  }

  return res.body.pipeThrough(cs);
}

async function getBase64Parser(version) {
  const cs = new CompressionStream('gzip');

  const stream = await getStreamFromRelease(version, cs);

  const reader = stream.getReader();

  let resultArr = [];

  while (true) {
    const read = await reader.read();

    if (read.done) break;

    resultArr = resultArr.concat(Array.from(read.value));
  }

  return Buffer.from(Uint8Array.from(resultArr).buffer).toString('base64');
}

function writeCompressedWasmParser(type) {
  return {
    name: 'write-wasm-parser',
    async closeBundle() {
      const base64Bytes = await getBase64Parser(config['sails-rs']);

      if (type === 'cjs') {
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

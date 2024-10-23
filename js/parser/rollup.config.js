import { writeFileSync, readFileSync, existsSync, rmSync } from 'fs';
import { execSync } from 'child_process';
import commonjs from '@rollup/plugin-commonjs';
import typescript from 'rollup-plugin-typescript2';

function checkParserFile() {
  return {
    name: 'check-parser-file',
    buildStart() {
      if (!existsSync('./parser.wasm')) {
        throw new Error('parser.wasm file not found');
      }
    },
  };
}

function compressParser(type) {
  return {
    name: 'compress-parser',
    async closeBundle() {
      const buf = readFileSync('./parser.wasm');

      const cs = new CompressionStream('gzip');

      const compressedReadableStream = new Response(buf).body.pipeThrough(cs);

      const reader = compressedReadableStream.getReader();

      let resultArr = [];

      while (true) {
        const read = await reader.read();

        if (read.done) break;

        resultArr = resultArr.concat(Array.from(read.value));
      }

      const base64Bytes = Buffer.from(Uint8Array.from(resultArr).buffer).toString('base64');

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
      checkParserFile(),
      cleanOldBuild(),
      typescript({
        tsconfig: 'tsconfig.build.json',
      }),
      compressParser('es'),
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
      compressParser('cjs'),
    ],
  },
];

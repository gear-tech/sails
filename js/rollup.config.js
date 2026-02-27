import { rmSync } from 'node:fs';
import commonjs from '@rollup/plugin-commonjs';
import nodeResolve from '@rollup/plugin-node-resolve';
import typescript from 'rollup-plugin-typescript2';
import dts from 'rollup-plugin-dts';

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
    input: ['src/index.ts', 'src/parser.ts', 'src/util.ts'],
    output: [
      {
        dir: 'lib',
        format: 'es',
        preserveModules: true,
        preserveModulesRoot: 'src',
      },
    ],
    plugins: [
      cleanOldBuild(),
      typescript({
        tsconfig: 'tsconfig.build.json',
      }),
      nodeResolve({
        preferBuiltins: true,
        resolveOnly: (module) => !module.includes('polkadot') && !module.includes('gear-js/api'),
      }),
    ],
  },
  {
    input: ['src/index.ts', 'src/parser.ts', 'src/util.ts'],
    output: [
      {
        dir: 'lib/cjs',
        format: 'cjs',
        entryFileNames: '[name].cjs',
        preserveModules: true,
        preserveModulesRoot: 'src',
        exports: 'named',
        strict: false,
      },
    ],
    plugins: [
      typescript({
        tsconfig: 'tsconfig.cjs.json',
      }),
      nodeResolve({
        preferBuiltins: true,
        resolveOnly: (module) => !module.includes('polkadot') && !module.includes('gear-js/api'),
      }),
      commonjs(),
    ],
  },
  // --- /parser (types) ---
  {
    input: 'node_modules/sails-js-parser-idl-v2/lib/index.d.ts',
    output: [{ file: 'lib/parser.d.ts', format: 'es' }],
    plugins: [nodeResolve({ extensions: ['.d.ts', '.ts'] }), dts()],
  },
  // --- /types (types) ---
  {
    input: 'node_modules/sails-js-types/lib/index.d.ts',
    output: [{ file: 'lib/types.d.ts', format: 'es' }],
    plugins: [nodeResolve({ extensions: ['.d.ts', '.ts'] }), dts()],
  },
  // --- /util (types) ---
  {
    input: 'node_modules/sails-js-util/lib/index.d.ts',
    output: [{ file: 'lib/util.d.ts', format: 'es' }],
    plugins: [nodeResolve({ extensions: ['.d.ts', '.ts'] }), dts()],
  },
];

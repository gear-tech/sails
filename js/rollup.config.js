import { rmSync } from 'fs';
import commonjs from '@rollup/plugin-commonjs';
import nodeResolve from '@rollup/plugin-node-resolve';
import typescript from 'rollup-plugin-typescript2';

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
    input: ['src/index.ts', 'src/app.ts'],
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
      nodeResolve({
        preferBuiltins: true,
        resolveOnly: (module) =>
          !module.includes('polkadot') &&
          !module.includes('gear-js/api') &&
          !module.includes('commander') &&
          !module.includes('sails-js-parser'),
      }),
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
      nodeResolve({
        preferBuiltins: true,
        resolveOnly: (module) =>
          !module.includes('polkadot') && !module.includes('gear-js/api') && !module.includes('commander'),
      }),
      commonjs(),
    ],
  },
];

import { rmSync } from 'fs';
import path from 'path';
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

const entries = ['x-bigint', 'x-fetch', 'x-global', 'x-randomvalues', 'x-textdecoder', 'x-textencoder', 'x-ws'].reduce(
  (all, p) => ({
    ...all,
    [`@polkadot/${p}`]: path.resolve(process.cwd(), `node_modules/@polkadot/${p}`),
  }),
  {},
);

export default [
  {
    input: 'src/app.ts',
    output: [{ file: 'build/app.js', format: 'module', preserveModules: false }],
    plugins: [
      cleanOldBuild(),
      typescript({ tsconfig: 'tsconfig.cjs.json' }),
      commonjs(),
      nodeResolve({ preferBuiltins: true, browser: true }),
    ],
  },
];

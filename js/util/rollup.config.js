import { rmSync } from 'node:fs';
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
    input: ['src/index.ts'],
    output: [
      {
        dir: 'lib',
        format: 'es',
        preserveModules: true,
      },
    ],
    plugins: [
      cleanOldBuild(),
      typescript({
        tsconfig: './tsconfig.json',
      }),
      nodeResolve({
        preferBuiltins: true,
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
      },
    ],
    plugins: [
      typescript({
        tsconfig: './tsconfig.json',
        tsconfigOverride: {
          compilerOptions: {
            declaration: false,
          },
        },
      }),
      nodeResolve({
        preferBuiltins: true,
      }),
      commonjs(),
    ],
  },
];

import { cpSync, writeFileSync } from 'fs';
import commonjs from '@rollup/plugin-commonjs';
import typescript from 'rollup-plugin-typescript2';

function writePackageJson(type) {
  return {
    name: 'write-package-json',
    closeBundle() {
      if (type === 'cjs') {
        writeFileSync('./lib/cjs/package.json', JSON.stringify({ type: 'commonjs' }));
      } else {
        cpSync('./package.json', 'lib/package.json');
      }
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
      typescript({
        tsconfig: 'tsconfig.build.json',
      }),
      writePackageJson('es'),
    ],
  },
  {
    input: 'src/index.ts',
    output: [
      {
        dir: 'lib/cjs',
        format: 'cjs',
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
      writePackageJson('cjs'),
    ],
  },
];

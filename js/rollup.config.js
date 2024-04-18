import { cpSync, writeFileSync } from 'fs';
import commonjs from '@rollup/plugin-commonjs';
import nodeResolve from '@rollup/plugin-node-resolve';
import typescript from 'rollup-plugin-typescript2';
import { strict } from 'assert';

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
function cpReadme() {
  return {
    name: 'cp-readme',
    closeBundle() {
      cpSync('./README.md', 'lib/README.md');
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
        banner: (chunk) => {
          if (chunk.fileName === 'app.js') {
            return '#!/usr/bin/env node';
          }
        },
      },
    ],
    plugins: [
      typescript({
        tsconfig: 'tsconfig.build.json',
      }),
      nodeResolve({
        preferBuiltins: true,
        resolveOnly: (module) =>
          !module.includes('polkadot') && !module.includes('gear-js/api') && !module.includes('commander'),
      }),
      writePackageJson('es'),
      cpReadme(),
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
      nodeResolve({
        preferBuiltins: true,
        resolveOnly: (module) =>
          !module.includes('polkadot') && !module.includes('gear-js/api') && !module.includes('commander'),
      }),
      commonjs(),
      writePackageJson('cjs'),
    ],
  },
  // {
  //   input: 'src/app.ts',
  //   output: {
  //     dir: 'lib',
  //     format: 'es',
  //     preserveModules: true,
  //     strict: false,
  //     banner: '#!/usr/bin/env node',
  //   },
  //   plugins: [
  //     typescript({
  //       tsconfig: 'tsconfig.build.json',
  //     }),
  //     nodeResolve({
  //       preferBuiltins: true,
  //       resolveOnly: (module) => !module.includes('polkadot') && !module.includes('commander'),
  //     }),
  //   ],
  // },
];

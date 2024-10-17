import { readFileSync, rmSync, writeFileSync } from 'fs';
import commonjs from '@rollup/plugin-commonjs';
import nodeResolve from '@rollup/plugin-node-resolve';
import typescript from 'rollup-plugin-typescript2';
import json from '@rollup/plugin-json';

function cleanOldBuild() {
  return {
    name: 'clean-old-build',
    buildStart() {
      rmSync('./build', { recursive: true, force: true });
    },
  };
}

function updateConfigVersions() {
  return {
    name: 'update-config-versions',
    buildStart() {
      const sailsJs = JSON.parse(readFileSync('../package.json', 'utf-8'));
      const rootPkgJson = JSON.parse(readFileSync('../../package.json', 'utf-8'));
      const config = JSON.parse(readFileSync('src/config.json', 'utf-8'));

      config.versions['gear-js'] = sailsJs.peerDependencies['@gear-js/api'];
      config.versions['polkadot-api'] = sailsJs.peerDependencies['@polkadot/api'];
      config.versions['sails-js'] = sailsJs.version;
      config.versions['typescript'] = rootPkgJson.devDependencies.typescript;

      writeFileSync('src/config.json', JSON.stringify(config, null, 2));
    },
  };
}

export default [
  {
    input: 'src/app.ts',
    output: [{ file: 'build/app.js', format: 'module', preserveModules: false }],
    plugins: [
      cleanOldBuild(),
      updateConfigVersions(),
      typescript({ tsconfig: 'tsconfig.json' }),
      commonjs(),
      json(),
      nodeResolve({ preferBuiltins: true, browser: true }),
    ],
  },
];

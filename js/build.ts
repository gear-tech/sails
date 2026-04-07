import { copyFileSync, rmSync } from 'node:fs';
import { buildEsmCjs, runTsc } from './build-utils';

const EXTERNAL = [
  '@gear-js/api',
  '@polkadot/api',
  '@polkadot/api/types',
  '@polkadot/types',
  '@polkadot/types/create',
  '@polkadot/types/types',
  '@polkadot/util',
];

rmSync('lib', { recursive: true, force: true });

await buildEsmCjs(['src/index.ts', 'src/parser.ts', 'src/util.ts'], EXTERNAL);

await runTsc(['-p', 'tsconfig.build.json', '--emitDeclarationOnly', '--outDir', 'lib']);
copyFileSync('types/lib/index.d.ts', 'lib/types.d.ts');

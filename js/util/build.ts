import { rmSync } from 'node:fs';
import { buildEsmCjs, runTsc } from '../build-utils';

rmSync('lib', { recursive: true, force: true });

await buildEsmCjs(['src/index.ts']);

await runTsc(['-p', 'tsconfig.json', '--emitDeclarationOnly', '--outDir', 'lib']);


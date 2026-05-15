import { rmSync } from 'node:fs';
import { runTsc } from '../build-utils';

rmSync('lib', { recursive: true, force: true });

await runTsc(['-p', 'tsconfig.json']);


import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';

// error TS1343: The 'import.meta' meta-property is only allowed when the '--module' option is
// 'es2020', 'es2022', 'esnext', 'system', 'node16', 'node18', 'node20', or 'nodenext'.
// const _dirname = dirname(fileURLToPath(import.meta.url));
const demoIdlPath = resolve(__dirname, '../../../../examples/demo/client/demo_client.idl');
const idl = readFileSync(demoIdlPath, 'utf8');
export default idl;

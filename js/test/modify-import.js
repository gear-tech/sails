import * as fs from 'fs';

const filesToModify = ['test/rmrk-catalog/lib.ts', 'test/rmrk-resource/lib.ts'];

for (const path of filesToModify) {
  const data = fs.readFileSync(path, 'utf8').replace(`from 'sails-js'`, `from '../../lib/index.js'`);
  fs.writeFileSync(path, data, 'utf8');
}

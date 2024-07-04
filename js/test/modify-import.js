import * as fs from 'fs';

const filesToModify = ['test/demo/lib.ts'];

for (const path of filesToModify) {
  const data = fs.readFileSync(path, 'utf8').replace(`from 'sails-js'`, `from '../../lib/index.js'`);
  fs.writeFileSync(path, data, 'utf8');
}

import * as fs from 'fs';
import { execSync } from 'child_process';

export default () => {
  execSync('rm -rf test/demo');

  // Build demo app
  execSync('cargo build -p demo --release');

  // Generate demo ts client
  execSync('node cli/build/app.js generate ../examples/demo/client/demo.idl -o ./test/demo --no-project --with-hooks');

  // Modify client imports

  const filesToModify = ['test/demo/lib.ts'];

  for (const path of filesToModify) {
    const data = fs.readFileSync(path, 'utf8').replace(`from 'sails-js'`, `from '../..'`);
    fs.writeFileSync(path, data, 'utf8');
  }
};

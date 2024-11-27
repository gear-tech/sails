import * as fs from 'fs';
import { execSync } from 'child_process';
import config from '../config.json' assert { type: 'json' };

const downloadAndWriteFile = async (fileName, writeTo) => {
  const link = `https://github.com/gear-tech/sails/releases/download/rs%2Fv${config['sails-rs']}/${fileName}`;

  const res = await fetch(link);
  if (!res.ok) {
    throw new Error(`Failed to fetch parser from ${link}`);
  }

  const blob = await res.blob();
  const buf = await blob.arrayBuffer();

  fs.writeFileSync(writeTo, Buffer.from(buf));
};

export default async () => {
  if (!fs.existsSync('test/demo')) {
    fs.mkdirSync('test/demo');
  }

  await Promise.all([
    downloadAndWriteFile('demo.wasm', 'test/demo/demo.wasm'),
    downloadAndWriteFile('demo.idl', 'test/demo/demo.idl'),
  ]);

  // Generate demo ts client
  execSync('node cli/build/app.js generate test/demo/demo.idl -o ./test/demo --no-project --yes');

  // Modify client imports
  const filesToModify = ['test/demo/lib.ts'];

  for (const path of filesToModify) {
    const data = fs.readFileSync(path, 'utf8').replace(`from 'sails-js'`, `from '../..'`);
    fs.writeFileSync(path, data, 'utf8');
  }
};

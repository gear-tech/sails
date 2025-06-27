import * as fs from 'node:fs';
import { execSync } from 'node:child_process';
import config from '../config.json' with { type: 'json' };

const USE_LOCAL_BUILD = process.env.USE_LOCAL_BUILD === 'true';

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

export default async function () {
  if (!fs.existsSync('test/demo')) {
    fs.mkdirSync('test/demo');
  }

  if (!USE_LOCAL_BUILD) {
    await Promise.all([
      downloadAndWriteFile('demo.wasm', 'test/demo/demo.wasm'),
      downloadAndWriteFile('demo.idl', 'test/demo/demo.idl'),
    ]);
  } else {
    fs.cpSync('../examples/demo/client/demo_client.idl', 'test/demo/demo.idl');
    fs.cpSync('../target/wasm32-gear/release/demo.opt.wasm', 'test/demo/demo.wasm');
  }

  // Generate demo ts client
  execSync('node cli/build/app.js generate test/demo/demo.idl -o ./test/demo --no-project --yes');

  // Modify client imports
  const filesToModify = ['test/demo/lib.ts'];

  for (const path of filesToModify) {
    const data = fs.readFileSync(path, 'utf8').replace(`from 'sails-js'`, `from '../..'`);
    fs.writeFileSync(path, data, 'utf8');
  }
}

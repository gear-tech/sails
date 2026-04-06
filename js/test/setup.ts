import * as fs from 'node:fs';
import config from '../config.json' with { type: 'json' };
import { Sails } from '..';
import { SailsIdlParser } from 'sails-js-parser';
import { ProjectBuilder } from '../cli/src/generate/index.ts';

const USE_LOCAL_BUILD = process.env.USE_LOCAL_BUILD === 'true';

async function downloadAndWriteFile(fileName: string, writeTo: string) {
  const link = `https://github.com/gear-tech/sails/releases/download/rs%2Fv${config['sails-rs']}/${fileName}`;
  const res = await fetch(link);

  if (!res.ok) {
    throw new Error(`Failed to fetch parser from ${link}`);
  }

  const blob = await res.blob();
  const buf = await blob.arrayBuffer();

  fs.writeFileSync(writeTo, Buffer.from(buf));
}

export async function prepareTestFixtures() {
  if (!fs.existsSync('test/demo')) {
    fs.mkdirSync('test/demo', { recursive: true });
  }

  if (USE_LOCAL_BUILD) {
    fs.cpSync('../examples/demo/client/demo_client.idl', 'test/demo/demo.idl');
    fs.cpSync('../target/wasm32-gear/release/demo.opt.wasm', 'test/demo/demo.wasm');
  } else {
    await Promise.all([
      downloadAndWriteFile('demo.wasm', 'test/demo/demo.wasm'),
      downloadAndWriteFile('demo.idl', 'test/demo/demo.idl'),
    ]);
  }

  const parser = await SailsIdlParser.new();
  const sails = new Sails(parser);

  const projectBuilder = new ProjectBuilder(sails, 'SailsProgram')
    .setRootPath('./test/demo')
    .setIdlPath('test/demo/demo.idl')
    .setIsProject(false)
    .setAutomaticOverride(true);

  await projectBuilder.build();

  const filesToModify = ['test/demo/lib.ts'];

  for (const path of filesToModify) {
    const data = fs.readFileSync(path, 'utf8').replace(`from 'sails-js'`, `from '../..'`);
    fs.writeFileSync(path, data, 'utf8');
  }
}

await prepareTestFixtures();

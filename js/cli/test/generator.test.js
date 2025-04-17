import { ProjectBuilder } from '../build/index.js';
import { Sails } from '../../lib';
import { SailsIdlParser } from '../../parser/lib';
import { fileURLToPath } from 'node:url';
import path from 'node:path';

const __filename = fileURLToPath(import.meta.url);

const demoIdlPath = path.join(__filename, '../../../../examples/demo/client/demo.idl');

console.log(demoIdlPath);

describe('generator', () => {
  test('demo lib', async () => {
    const parser = new SailsIdlParser();
    await parser.init();
    const sails = new Sails(parser);

    const generator = new ProjectBuilder(sails, 'program').setIdlPath(demoIdlPath);

    const lib = generator.generateLib();
    expect(lib).toMatchSnapshot();

    const types = generator.generateTypes();
    expect(types).toMatchSnapshot();
  });
});

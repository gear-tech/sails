import { ProjectBuilder } from '../build/index.js';
import { Sails } from '../../lib';
import { SailsIdlParser } from '../../parser/lib';

describe('generator', () => {
  test('demo lib', async () => {
    const parser = new SailsIdlParser();
    await parser.init();
    const sails = new Sails(parser);

    const generator = new ProjectBuilder(sails, 'program').setIdlPath('../../examples/demo/client/demo.idl');

    const lib = generator.generateLib();
    expect(lib).toMatchSnapshot();

    const types = generator.generateTypes();
    expect(types).toMatchSnapshot();
  });
});

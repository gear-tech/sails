import { SailsIdlParser } from '..';

describe('parser-v2 error handling', () => {
  test('throws on invalid pointer from wasm', async () => {
    const parser: any = new SailsIdlParser();

    parser._memory = new WebAssembly.Memory({ initial: 1 });
    parser._exports = {
      parse_idl_to_json: () => parser._memory.buffer.byteLength + 1,
      free_parse_result: jest.fn(),
    };
    parser._instance = { exports: parser._exports };

    expect(() => parser.parse('service TestService {}')).toThrow(
      'Invalid pointer returned from WASM parse_idl_to_json',
    );
  });

  test('throws on invalid IDL', async () => {
    const parser: any = new SailsIdlParser();
    await parser.init();

    expect(() => parser.parse('unknown TestService {}')).toThrow(`Error code: 1, Error details:  --> 1:1
  |
1 | unknown TestService {}
  | ^---
  |
  = expected Top`,
    );
  });

  test('rejects self-extending services with a validation error', async () => {
    const parser = new SailsIdlParser();
    await parser.init();

    expect(() =>
      parser.parse(`
        service A {
          extends { A }
          functions { Ping() -> bool; }
        }
      `),
    ).toThrow(/cyclic/);
  });

  test('rejects extends cycles with a validation error', async () => {
    const parser = new SailsIdlParser();
    await parser.init();

    expect(() =>
      parser.parse(`
        service A {
          extends { B }
          functions { Ping() -> bool; }
        }
        service B {
          extends { A }
          functions { Pong() -> bool; }
        }
      `),
    ).toThrow(/cyclic/);
  });

  test('rejects duplicate service names with a validation error', async () => {
    const parser = new SailsIdlParser();
    await parser.init();

    expect(() =>
      parser.parse(`
        service S {
          functions { A() -> bool; }
        }
        service S {
          functions { B() -> bool; }
        }
      `),
    ).toThrow(/duplicate/);
  });
});

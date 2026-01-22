describe('parser-v2 error handling', () => {
  test('throws on invalid pointer from wasm', async () => {
    const { SailsIdlParser } = await import('../src/parser.js');
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
    const { SailsIdlParser } = await import('../src/parser.js');
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
});

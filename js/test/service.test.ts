import { parseSailsIdl } from '../src';

describe('service', () => {
  test('simple service', () => {
    const text = `service {
      async DoThis : (a1: string) -> u8;
    }`;

    const result = parseSailsIdl(text);

    expect(result.services).toHaveLength(1);
    expect(result.services[0].methods).toHaveLength(1);
    expect(result.services[0].methods[0]).toEqual({
      kind: 'message',
      def: {
        name: 'DoThis',
        args: [
          {
            name: 'a1',
            type: {
              kind: 'typeName',
              def: {
                kind: 'simple',
                name: 'string',
              },
            },
          },
        ],
        output: {
          kind: 'typeName',
          def: {
            kind: 'simple',
            name: 'u8',
          },
        },
      },
    });
  });

  test('service with multiple methods', () => {
    const text = `service {
      async DoThis : (a1: u32, a2: struct { string, opt u8 }) -> result ( string, u8 );
      async DoThat : (a1: vec u8) -> string;
      query GetThis : (a1: string) -> u8;
      query GetThat : (a1: vec u8) -> string;
    }`;

    const result = parseSailsIdl(text);

    expect(result.services).toHaveLength(1);
    expect(result.services[0].methods).toHaveLength(4);

    expect(result.services[0].methods[0]).toEqual({
      kind: 'message',
      def: {
        name: 'DoThis',
        args: [
          {
            name: 'a1',
            type: {
              kind: 'typeName',
              def: {
                kind: 'simple',
                name: 'u32',
              },
            },
          },
          {
            name: 'a2',
            type: {
              kind: 'tuple',
              def: {
                fields: [
                  {
                    kind: 'typeName',
                    def: {
                      kind: 'simple',
                      name: 'string',
                    },
                  },
                  {
                    kind: 'option',
                    def: {
                      kind: 'typeName',
                      def: {
                        kind: 'simple',
                        name: 'u8',
                      },
                    },
                  },
                ],
              },
            },
          },
        ],
        output: {
          kind: 'result',
          def: {
            ok: {
              kind: 'typeName',
              def: {
                kind: 'simple',
                name: 'string',
              },
            },
            err: {
              kind: 'typeName',
              def: {
                kind: 'simple',
                name: 'u8',
              },
            },
          },
        },
      },
    });

    expect(result.services[0].methods[1]).toEqual({
      kind: 'message',
      def: {
        name: 'DoThat',
        args: [
          {
            name: 'a1',
            type: {
              kind: 'vec',
              def: {
                kind: 'typeName',
                def: {
                  kind: 'simple',
                  name: 'u8',
                },
              },
            },
          },
        ],
        output: {
          kind: 'typeName',
          def: {
            kind: 'simple',
            name: 'string',
          },
        },
      },
    });

    expect(result.services[0].methods[2]).toEqual({
      kind: 'query',
      def: {
        name: 'GetThis',
        args: [
          {
            name: 'a1',
            type: {
              kind: 'typeName',
              def: {
                kind: 'simple',
                name: 'string',
              },
            },
          },
        ],
        output: {
          kind: 'typeName',
          def: {
            kind: 'simple',
            name: 'u8',
          },
        },
      },
    });

    expect(result.services[0].methods[3]).toEqual({
      kind: 'query',
      def: {
        name: 'GetThat',
        args: [
          {
            name: 'a1',
            type: {
              kind: 'vec',
              def: {
                kind: 'typeName',
                def: {
                  kind: 'simple',
                  name: 'u8',
                },
              },
            },
          },
        ],
        output: {
          kind: 'typeName',
          def: {
            kind: 'simple',
            name: 'string',
          },
        },
      },
    });
  });
});

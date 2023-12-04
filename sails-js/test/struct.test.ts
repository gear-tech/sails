import { parseSailsIdl } from '../src';

describe('struct', () => {
  test('simple struct', () => {
    const text = `type SimpleStruct = struct {
        a: string,
        b: u32,
      }
    `;
    const result = parseSailsIdl(text);

    expect(result).toEqual({
      services: [],
      types: [
        {
          def: {
            fields: [
              {
                type: {
                  def: {
                    kind: 'simple',
                    name: 'string',
                  },
                  kind: 'typeName',
                },
                name: 'a',
              },
              {
                type: {
                  def: {
                    kind: 'simple',
                    name: 'u32',
                  },
                  kind: 'typeName',
                },
                name: 'b',
              },
            ],
          },
          kind: 'struct',
          type: {
            kind: 'simple',
            name: 'SimpleStruct',
          },
        },
      ],
    });
  });

  test('struct with option', () => {
    const text = `type StructWithOption = struct {
        a: opt string,
        b: u32,
      }
    `;
    const result = parseSailsIdl(text);

    expect(result).toEqual({
      services: [],
      types: [
        {
          def: {
            fields: [
              {
                type: {
                  def: {
                    def: {
                      kind: 'simple',
                      name: 'string',
                    },
                    kind: 'typeName',
                  },
                  kind: 'option',
                },
                name: 'a',
              },
              {
                type: {
                  def: {
                    kind: 'simple',
                    name: 'u32',
                  },
                  kind: 'typeName',
                },
                name: 'b',
              },
            ],
          },
          kind: 'struct',
          type: {
            kind: 'simple',
            name: 'StructWithOption',
          },
        },
      ],
    });
  });

  test('struct with result', () => {
    const text = `type StructWithResult = struct {
        a: result (string, u32),
        b: u32,
      }
    `;
    const result = parseSailsIdl(text);

    expect(result).toEqual({
      services: [],
      types: [
        {
          def: {
            fields: [
              {
                type: {
                  def: {
                    ok: {
                      def: {
                        kind: 'simple',
                        name: 'string',
                      },
                      kind: 'typeName',
                    },
                    err: {
                      def: {
                        kind: 'simple',
                        name: 'u32',
                      },
                      kind: 'typeName',
                    },
                  },
                  kind: 'result',
                },
                name: 'a',
              },
              {
                type: {
                  def: {
                    kind: 'simple',
                    name: 'u32',
                  },
                  kind: 'typeName',
                },
                name: 'b',
              },
            ],
          },
          kind: 'struct',
          type: {
            kind: 'simple',
            name: 'StructWithResult',
          },
        },
      ],
    });
  });

  test('struct with tuple', () => {
    const text = `type StructWithTuple = struct {
      a: struct { string, u32 },
      b: u32
    }`;

    const result = parseSailsIdl(text);

    expect(result).toEqual({
      services: [],
      types: [
        {
          def: {
            fields: [
              {
                type: {
                  def: {
                    fields: [
                      {
                        def: {
                          kind: 'simple',
                          name: 'string',
                        },
                        kind: 'typeName',
                      },
                      {
                        def: {
                          kind: 'simple',
                          name: 'u32',
                        },
                        kind: 'typeName',
                      },
                    ],
                  },
                  kind: 'tuple',
                },
                name: 'a',
              },
              {
                type: {
                  def: {
                    kind: 'simple',
                    name: 'u32',
                  },
                  kind: 'typeName',
                },
                name: 'b',
              },
            ],
          },
          kind: 'struct',
          type: {
            kind: 'simple',
            name: 'StructWithTuple',
          },
        },
      ],
    });
  });

  test('struct with vec', () => {
    const text = `type StructWithVec = struct {
      a: vec string,
      b: u32
    }`;

    const result = parseSailsIdl(text);

    expect(result.types).toHaveLength(1);
    expect(result.types[0].kind).toBe('struct');

    expect(result.types[0]).toEqual({
      def: {
        fields: [
          {
            type: {
              def: {
                def: {
                  kind: 'simple',
                  name: 'string',
                },
                kind: 'typeName',
              },
              kind: 'vec',
            },
            name: 'a',
          },
          {
            type: {
              def: {
                kind: 'simple',
                name: 'u32',
              },
              kind: 'typeName',
            },
            name: 'b',
          },
        ],
      },
      kind: 'struct',
      type: {
        kind: 'simple',
        name: 'StructWithVec',
      },
    });
  });

  test('generic struct', () => {
    const text = `type GenericStruct<u32, opt result (vec u8, struct { u8, u32 })> = struct {
      a: u32,
      b: opt result (vec u8, struct { u8, u32 })
    }`;

    const result = parseSailsIdl(text);

    expect(result.types[0].kind).toBe('struct');

    expect(result.types[0].type).toEqual({
      generic: [
        { def: { kind: 'simple', name: 'u32' }, kind: 'typeName' },
        {
          def: {
            def: {
              ok: {
                def: {
                  def: { name: 'u8', kind: 'simple' },
                  kind: 'typeName',
                },
                kind: 'vec',
              },
              err: {
                def: {
                  fields: [
                    { def: { name: 'u8', kind: 'simple' }, kind: 'typeName' },
                    { def: { name: 'u32', kind: 'simple' }, kind: 'typeName' },
                  ],
                },
                kind: 'tuple',
              },
            },
            kind: 'result',
          },
          kind: 'option',
        },
      ],
      kind: 'generic',
      name: 'GenericStruct',
    });

    expect(result.types[0].def).toEqual({
      fields: [
        { name: 'a', type: { kind: 'typeName', def: { kind: 'simple', name: 'u32' } } },
        {
          name: 'b',
          type: {
            def: {
              def: {
                ok: {
                  def: {
                    def: { name: 'u8', kind: 'simple' },
                    kind: 'typeName',
                  },
                  kind: 'vec',
                },
                err: {
                  def: {
                    fields: [
                      { def: { name: 'u8', kind: 'simple' }, kind: 'typeName' },
                      { def: { name: 'u32', kind: 'simple' }, kind: 'typeName' },
                    ],
                  },
                  kind: 'tuple',
                },
              },
              kind: 'result',
            },
            kind: 'option',
          },
        },
      ],
    });
  });
});

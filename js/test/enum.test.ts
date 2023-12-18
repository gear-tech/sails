import { parseSailsIdl } from '../src';

describe('enum', () => {
  test('simple enum', () => {
    const text = `type SimpleEnum = enum {
        One,
        Two,
        Three,
    }`;

    const result = parseSailsIdl(text);

    expect(result.types).toHaveLength(1);

    expect(result.types[0]).toEqual({
      def: {
        variants: [
          {
            name: 'One',
          },
          {
            name: 'Two',
          },
          {
            name: 'Three',
          },
        ],
      },
      kind: 'enum',
      type: {
        kind: 'simple',
        name: 'SimpleEnum',
      },
    });
  });

  test('complex enum', () => {
    const text = `type ComplexEnum = enum {
        One,
        Two: u32,
        Three: opt vec u8,
        Four: struct { a: u32, b: opt u16 },
        Five: struct { string, u32 },
        Six: struct { u32 },
    }`;

    const result = parseSailsIdl(text);

    expect(result.types).toHaveLength(1);

    expect(result.types[0].kind).toBe('enum');
    expect(result.types[0].type).toEqual({ name: 'ComplexEnum', kind: 'simple' });

    expect(result.types[0].def.variants).toHaveLength(6);

    expect(result.types[0].def.variants).toEqual([
      { name: 'One' },
      { name: 'Two', type: { kind: 'typeName', def: { kind: 'simple', name: 'u32' } } },
      {
        name: 'Three',
        type: {
          kind: 'option',
          def: {
            kind: 'vec',
            def: {
              kind: 'typeName',
              def: { kind: 'simple', name: 'u8' },
            },
          },
        },
      },
      {
        name: 'Four',
        type: {
          kind: 'struct',
          def: {
            fields: [
              { name: 'a', type: { kind: 'typeName', def: { kind: 'simple', name: 'u32' } } },
              {
                name: 'b',
                type: {
                  kind: 'option',
                  def: {
                    kind: 'typeName',
                    def: { kind: 'simple', name: 'u16' },
                  },
                },
              },
            ],
          },
        },
      },
      {
        name: 'Five',
        type: {
          kind: 'tuple',
          def: {
            fields: [
              { kind: 'typeName', def: { kind: 'simple', name: 'string' } },
              {
                kind: 'typeName',
                def: { kind: 'simple', name: 'u32' },
              },
            ],
          },
        },
      },
      {
        name: 'Six',
        type: { kind: 'tuple', def: { fields: [{ kind: 'typeName', def: { kind: 'simple', name: 'u32' } }] } },
      },
    ]);
  });
});

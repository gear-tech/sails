import { parseSailsIdl } from '../src';

describe('variant', () => {
  test('simple variant', () => {
    const text = `type SimpleVariant = variant {
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
      kind: 'variant',
      type: {
        kind: 'simple',
        name: 'SimpleVariant',
      },
    });
  });

  test('complex variant', () => {
    const text = `type ComplexVariant = variant {
        One,
        Two: u32,
        Three: opt vec u8,
        Four: struct { a: u32, b: opt u16 },
        Five: struct { string, u32 },
        Six: struct { u32 },
    }`;

    const result = parseSailsIdl(text);

    expect(result.types).toHaveLength(1);

    expect(result.types[0].kind).toBe('variant');
    expect(result.types[0].type).toEqual({ name: 'ComplexVariant', kind: 'simple' });

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

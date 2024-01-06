import { parseSailsIdl } from '../src';

describe('alias', () => {
  test('simple alias', () => {
    const text = 'type Alias = string';
    const result = parseSailsIdl(text);
    expect(result).toEqual({
      services: [],
      types: [
        {
          def: { name: 'string', kind: 'simple' },
          kind: 'typeName',
          type: { kind: 'simple', name: 'Alias' },
        },
      ],
    });
  });

  test('option alias', () => {
    const text = 'type Alias = opt string';
    const result = parseSailsIdl(text);

    expect(result).toEqual({
      services: [],
      types: [
        {
          def: {
            def: { kind: 'simple', name: 'string' },
            kind: 'typeName',
          },
          kind: 'option',
          type: { kind: 'simple', name: 'Alias' },
        },
      ],
    });
  });

  test('result alias', () => {
    const text = 'type Alias = result (string, u32)';
    const result = parseSailsIdl(text);

    expect(result).toEqual({
      services: [],
      types: [
        {
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
          type: { kind: 'simple', name: 'Alias' },
        },
      ],
    });
  });

  test('vec alias', () => {
    const text = 'type Alias = vec string';
    const result = parseSailsIdl(text);

    expect(result).toEqual({
      services: [],
      types: [
        {
          def: {
            def: {
              kind: 'simple',
              name: 'string',
            },
            kind: 'typeName',
          },
          kind: 'vec',
          type: { kind: 'simple', name: 'Alias' },
        },
      ],
    });
  });
});

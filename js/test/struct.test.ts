import { Sails } from '../lib';

let sails: Sails;

beforeAll(async () => {
  sails = await Sails.new();
});

describe('struct', () => {
  test('simple struct', () => {
    const text = `type SimpleStruct = struct {
        a: str,
        b: u32,
      };

      service {}
    `;
    const result = sails.parseIdl(text);

    expect(result.scaleCodecTypes).toEqual({
      SimpleStruct: {
        a: 'String',
        b: 'u32',
      },
    });

    expect(result.functions).toEqual({});

    const encoded = result.registry.createType('SimpleStruct', { a: 'hello', b: 123 });

    expect(encoded.toJSON()).toEqual({
      a: 'hello',
      b: 123,
    });
  });

  test('struct with option', () => {
    const text = `type StructWithOption = struct {
        a: opt str,
        b: u32,
      };

      service {}
    `;
    const result = sails.parseIdl(text);

    expect(result.scaleCodecTypes).toEqual({
      StructWithOption: {
        a: 'Option<String>',
        b: 'u32',
      },
    });
    expect(result.functions).toEqual({});

    let encoded = result.registry.createType('StructWithOption', { a: 'hello', b: 123 });

    expect(encoded.toJSON()).toEqual({
      a: 'hello',
      b: 123,
    });

    encoded = result.registry.createType('StructWithOption', { a: null, b: 123 });

    expect(encoded.toJSON()).toEqual({
      a: null,
      b: 123,
    });
  });

  test('struct with result', () => {
    const text = `type StructWithResult = struct {
        a: result (str, u32),
        b: u32,
      };

      service {}
    `;
    const result = sails.parseIdl(text);

    expect(result.scaleCodecTypes).toEqual({
      StructWithResult: {
        a: 'Result<String, u32>',
        b: 'u32',
      },
    });

    expect(result.functions).toEqual({});

    let encoded = result.registry.createType('StructWithResult', { a: { ok: 'hello' }, b: 123 });

    expect(encoded.toJSON()).toEqual({
      a: { ok: 'hello' },
      b: 123,
    });

    encoded = result.registry.createType('StructWithResult', { a: { err: 123 }, b: 123 });

    expect(encoded.toJSON()).toEqual({
      a: { err: 123 },
      b: 123,
    });
  });

  test('struct with tuple', () => {
    const text = `type StructWithTuple = struct {
      a: struct { str, u32 },
      b: u32
    };
    
    service {}`;

    const result = sails.parseIdl(text);

    expect(result.scaleCodecTypes).toEqual({
      StructWithTuple: {
        a: '(String, u32)',
        b: 'u32',
      },
    });

    expect(result.functions).toEqual({});

    let encoded = result.registry.createType('StructWithTuple', { a: ['hello', 123], b: 123 });
    expect(encoded.toJSON()).toEqual({
      a: ['hello', 123],
      b: 123,
    });
  });

  test('struct with vec', () => {
    const text = `type StructWithVec = struct {
      a: vec str,
      b: u32
    };

    service {}`;

    const result = sails.parseIdl(text);

    expect(result.scaleCodecTypes).toEqual({
      StructWithVec: {
        a: 'Vec<String>',
        b: 'u32',
      },
    });

    expect(result.functions).toEqual({});

    let encoded = result.registry.createType('StructWithVec', { a: ['hello', 'world'], b: 123 });

    expect(encoded.toJSON()).toEqual({
      a: ['hello', 'world'],
      b: 123,
    });
  });

  test('struct with fixed size array', () => {
    const text = `type StructWithArray = struct {
      a: [u32, 3],
      b: u32
    };

    service {}`;

    const result = sails.parseIdl(text);

    expect(result.scaleCodecTypes).toEqual({
      StructWithArray: {
        a: '[u32; 3]',
        b: 'u32',
      },
    });

    expect(result.functions).toEqual({});

    let encoded = result.registry.createType('StructWithArray', { a: [1, 2, 3], b: 123 });

    expect(encoded.toJSON()).toEqual({
      a: [1, 2, 3],
      b: 123,
    });
  });

  test('struct with map', () => {
    const text = `type StructWithMap = struct {
      a: map (str, u32),
      b: u32
    };

    service {}`;

    const result = sails.parseIdl(text);

    expect(result.scaleCodecTypes).toEqual({
      StructWithMap: {
        a: 'BTreeMap<String, u32>',
        b: 'u32',
      },
    });

    expect(result.functions).toEqual({});

    let encoded = result.registry.createType('StructWithMap', { a: { foo: 123, bar: 456 }, b: 123 });

    expect(encoded.toJSON()).toEqual({
      a: { foo: 123, bar: 456 },
      b: 123,
    });
  });
});

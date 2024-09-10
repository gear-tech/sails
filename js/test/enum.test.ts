import { SailsIdlParser } from 'sails-js-parser';
import { Sails } from '..';

let sails: Sails;

beforeAll(async () => {
  const parser = await SailsIdlParser.new();
  sails = new Sails(parser);
});

describe('enum', () => {
  test('simple enum', () => {
    const idl = `type SimpleEnum = enum {
        One,
        Two,
        Three,
    };

    service TestService {}`;

    sails.parseIdl(idl);

    expect(sails.scaleCodecTypes).toEqual({
      SimpleEnum: { _enum: ['One', 'Two', 'Three'] },
    });

    expect(sails.registry.createType('SimpleEnum', 'One').toU8a()[0]).toBe(0);
    expect(sails.registry.createType('SimpleEnum', 'One').toJSON()).toEqual('One');
    expect(sails.registry.createType('SimpleEnum', 'Two').toU8a()[0]).toBe(1);
    expect(sails.registry.createType('SimpleEnum', 'Two').toJSON()).toEqual('Two');
    expect(sails.registry.createType('SimpleEnum', 'Three').toU8a()[0]).toBe(2);
    expect(sails.registry.createType('SimpleEnum', 'Three').toJSON()).toEqual('Three');
  });

  test('complex enum', () => {
    const text = `type ComplexEnum = enum {
        One,
        Two: u32,
        Three: opt vec u8,
        Four: struct { a: u32, b: opt u16 },
        Five: struct { str, u32 },
        Six: [map (str, u32), 3],
    };

    service TestService {}`;

    const result = sails.parseIdl(text);

    expect(result.scaleCodecTypes).toEqual({
      ComplexEnum: {
        _enum: {
          One: 'Null',
          Two: 'u32',
          Three: 'Option<Vec<u8>>',
          Four: { a: 'u32', b: 'Option<u16>' },
          Five: '(String, u32)',
          Six: '[BTreeMap<String, u32>; 3]',
        },
      },
    });

    expect(result.registry.createType('ComplexEnum', 'One').toU8a()[0]).toBe(0);
    expect(result.registry.createType('ComplexEnum', 'One').toJSON()).toEqual({ one: null });
    expect(result.registry.createType('ComplexEnum', { Two: 123 }).toU8a()[0]).toBe(1);
    expect(result.registry.createType('ComplexEnum', { Two: 123 }).toJSON()).toEqual({
      two: 123,
    });
    expect(result.registry.createType('ComplexEnum', { Three: null }).toU8a()[0]).toBe(2);
    expect(result.registry.createType('ComplexEnum', { Three: null }).toJSON()).toEqual({
      three: null,
    });
    expect(result.registry.createType('ComplexEnum', { Three: [1, 2, 3] }).toU8a()[0]).toBe(2);
    expect(result.registry.createType('ComplexEnum', { Three: '0x1234' }).toJSON()).toEqual({
      three: '0x1234',
    });
    expect(result.registry.createType('ComplexEnum', { Four: { a: 123, b: null } }).toU8a()[0]).toBe(3);
    expect(result.registry.createType('ComplexEnum', { Four: { a: 123, b: null } }).toJSON()).toEqual({
      four: { a: 123, b: null },
    });
    expect(result.registry.createType('ComplexEnum', { Four: { a: 123, b: 456 } }).toU8a()[0]).toBe(3);
    expect(result.registry.createType('ComplexEnum', { Four: { a: 123, b: 456 } }).toJSON()).toEqual({
      four: { a: 123, b: 456 },
    });
    expect(result.registry.createType('ComplexEnum', { Five: ['abc', 123] }).toU8a()[0]).toBe(4);
    expect(result.registry.createType('ComplexEnum', { Five: ['abc', 123] }).toJSON()).toEqual({
      five: ['abc', 123],
    });
    expect(
      result.registry.createType('ComplexEnum', { Six: [{ foo: 1 }, { bar: 2 }, { foobar: 3 }] }).toJSON(),
    ).toEqual({ six: [{ foo: 1 }, { bar: 2 }, { foobar: 3 }] });
  });
});

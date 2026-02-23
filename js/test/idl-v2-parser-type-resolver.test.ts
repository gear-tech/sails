import { SailsIdlParser } from 'sails-js-parser-idl-v2';

import { SailsProgram } from '..';

let parser: SailsIdlParser;

beforeAll(async () => {
  // Initialize Sails parser
  parser = new SailsIdlParser();
  await parser.init();
});

describe('type-resolver-v2 structs', () => {
  test('struct simple', () => {
    const text = `
      program Test {
        types {
          struct SimpleStruct {
            a: String,
            b: u32,
          }
        }
        constructors {
          Create(p: SimpleStruct);
        }
      }
    `;
    const program = new SailsProgram(parser.parse(text));
    expect(program.ctors).toBeDefined();

    const encoded = program.registry.createType('SimpleStruct', { a: 'hello', b: 123 });
    expect(encoded.toJSON()).toEqual({
      a: 'hello',
      b: 123,
    });
  });

  test('struct with option', () => {
    const text = `
      program Test {
        types {
          struct StructWithOption {
            a: Option<String>,
            b: u32,
          }
        }
        constructors {
          Create(p: StructWithOption);
        }
      }
    `;
    const program = new SailsProgram(parser.parse(text));
    expect(program.ctors).toBeDefined();

    let encoded = program.registry.createType('StructWithOption', { a: 'hello', b: 123 });
    expect(encoded.toJSON()).toEqual({
      a: 'hello',
      b: 123,
    });

    encoded = program.registry.createType('StructWithOption', { a: null, b: 123 });
    expect(encoded.toJSON()).toEqual({
      a: null,
      b: 123,
    });
  });

  test('struct with result', () => {
    const text = `
      program Test {
        types {
          struct StructWithResult {
            a: Result<String, u32>,
            b: u32,
          }
        }
        constructors {
          Create(p: StructWithResult);
        }
      }
    `;
    const program = new SailsProgram(parser.parse(text));
    expect(program.ctors).toBeDefined();

    let encoded = program.registry.createType('StructWithResult', { a: { ok: 'hello' }, b: 123 });
    expect(encoded.toJSON()).toEqual({
      a: { ok: 'hello' },
      b: 123,
    });

    encoded = program.registry.createType('StructWithResult', { a: { err: 123 }, b: 123 });
    expect(encoded.toJSON()).toEqual({
      a: { err: 123 },
      b: 123,
    });
  });

  test('struct with tuple', () => {
    const text = `
      program Test {
        types {
          struct StructWithTuple {
            a: (String, u32),
            b: u32,
          }
        }
        constructors {
          Create(p: StructWithTuple);
        }
      }
    `;
    const program = new SailsProgram(parser.parse(text));
    expect(program.ctors).toBeDefined();

    const encoded = program.registry.createType('StructWithTuple', { a: ['hello', 123], b: 123 });
    expect(encoded.toJSON()).toEqual({
      a: ['hello', 123],
      b: 123,
    });
  });

  test('struct with vec', () => {
    const text = `
      program Test {
        types {
          struct StructWithVec {
            a: [String],
            b: u32,
          }
        }
        constructors {
          Create(p: StructWithVec);
        }
      }
    `;
    const program = new SailsProgram(parser.parse(text));
    expect(program.ctors).toBeDefined();

    const encoded = program.registry.createType('StructWithVec', { a: ['hello', 'world'], b: 123 });
    expect(encoded.toJSON()).toEqual({
      a: ['hello', 'world'],
      b: 123,
    });
  });

  test('struct with fixed size array', () => {
    const text = `
      program Test {
        types {
          struct StructWithArray {
            a: [u32; 3],
            b: u32,
          }
        }
        constructors {
          Create(p: StructWithArray);
        }
      }
    `;
    const program = new SailsProgram(parser.parse(text));
    expect(program.ctors).toBeDefined();

    const encoded = program.registry.createType('StructWithArray', { a: [1, 2, 3], b: 123 });
    expect(encoded.toJSON()).toEqual({
      a: [1, 2, 3],
      b: 123,
    });
  });
});

describe('type-resolver-v2 enums', () => {
  test('simple enum', () => {
    const text = `
      program Test {
        types {
          enum SimpleEnum {
            One,
            Two,
            Three,
          }
        }
        constructors {
          Create(p: SimpleEnum);
        }
      }
    `;
    const program = new SailsProgram(parser.parse(text));
    expect(program.ctors).toBeDefined();

    expect(program.registry.createType('SimpleEnum', 'One').toU8a()[0]).toBe(0);
    expect(program.registry.createType('SimpleEnum', 'One').toJSON()).toEqual('One');
    expect(program.registry.createType('SimpleEnum', 'Two').toU8a()[0]).toBe(1);
    expect(program.registry.createType('SimpleEnum', 'Two').toJSON()).toEqual('Two');
    expect(program.registry.createType('SimpleEnum', 'Three').toU8a()[0]).toBe(2);
    expect(program.registry.createType('SimpleEnum', 'Three').toJSON()).toEqual('Three');
  });

  test('complex enum', () => {
    const text = `
      program Test {
        types {
          enum ComplexEnum {
            One,
            Two(u32),
            Three(Option<[u8]>),
            Four { a: u32, b: Option<u16> },
            Five(String, u32),
            Six([(String, u32); 3]),
          }
        }
        constructors {
          Create(p: ComplexEnum);
        }
      }
    `;
    const program = new SailsProgram(parser.parse(text));
    expect(program.ctors).toBeDefined();

    expect(program.registry.createType('ComplexEnum', 'One').toU8a()[0]).toBe(0);
    expect(program.registry.createType('ComplexEnum', 'One').toJSON()).toEqual({ one: null });
    expect(program.registry.createType('ComplexEnum', { Two: 123 }).toU8a()[0]).toBe(1);
    expect(program.registry.createType('ComplexEnum', { Two: 123 }).toJSON()).toEqual({
      two: 123,
    });
    expect(program.registry.createType('ComplexEnum', { Three: null }).toU8a()[0]).toBe(2);
    expect(program.registry.createType('ComplexEnum', { Three: null }).toJSON()).toEqual({
      three: null,
    });
    expect(program.registry.createType('ComplexEnum', { Three: [1, 2, 3] }).toU8a()[0]).toBe(2);
    expect(program.registry.createType('ComplexEnum', { Three: '0x1234' }).toJSON()).toEqual({
      three: '0x1234',
    });
    expect(program.registry.createType('ComplexEnum', { Four: { a: 123, b: null } }).toU8a()[0]).toBe(3);
    expect(program.registry.createType('ComplexEnum', { Four: { a: 123, b: null } }).toJSON()).toEqual({
      four: { a: 123, b: null },
    });
    expect(program.registry.createType('ComplexEnum', { Four: { a: 123, b: 456 } }).toU8a()[0]).toBe(3);
    expect(program.registry.createType('ComplexEnum', { Four: { a: 123, b: 456 } }).toJSON()).toEqual({
      four: { a: 123, b: 456 },
    });
    expect(program.registry.createType('ComplexEnum', { Five: ['abc', 123] }).toU8a()[0]).toBe(4);
    expect(program.registry.createType('ComplexEnum', { Five: ['abc', 123] }).toJSON()).toEqual({
      five: ['abc', 123],
    });
    expect(
      program.registry
        .createType('ComplexEnum', { Six: [['foo', 1], ['bar', 2], ['foobar', 3]] })
        .toJSON(),
    ).toEqual({ six: [['foo', 1], ['bar', 2], ['foobar', 3]] });
  });
});



describe('type-resolver-v2 generics', () => {
  test('generic struct and enum', () => {
    const text = `
      program Test {
        types {
          struct Tuple<T>(T, Option<T>);

          struct Array<U>([U; 4]);

          enum GenericEnum<T, U> {
            One,
            Two(T),
            Three { p1: T, p2: Option<U> },
          }
        }
        constructors {
          Create(
            p1: GenericEnum<u8, String>,
            p2: GenericEnum<Option<u8>, String>,
            p3: GenericEnum<Tuple<u8>, Array<String>>,
            p4: GenericEnum<Array<Tuple<u8>>, Tuple<Array<String>>>,
          );
        }
      }
    `;
    const idlDoc = parser.parse(text);
    const program = new SailsProgram(idlDoc);
    expect(program.ctors).toBeDefined();

    const tupleU8 = [7, null];
    const tupleU8Alt = [8, 9];
    const arrayTupleU8 = Array.from({ length: 4 }, (_, i) => [i, null]);

    expect(program.registry.createType('GenericEnum<u8,String>', 'One').toJSON()).toEqual({ one: null });
    expect(program.registry.createType('GenericEnum<u8,String>', { Two: 7 }).toJSON()).toEqual({ two: 7 });
    expect(
      program.registry.createType('GenericEnum<u8,String>', { Three: { p1: 7, p2: 'hello' } }).toJSON(),
    ).toEqual({ three: { p1: 7, p2: 'hello' } });

    expect(program.registry.createType('GenericEnum<Option<u8>,String>', 'One').toJSON()).toEqual({ one: null });
    expect(program.registry.createType('GenericEnum<Option<u8>,String>', { Two: 7 }).toJSON()).toEqual({ two: 7 });
    expect(
      program.registry
        .createType('GenericEnum<Option<u8>,String>', { Three: { p1: null, p2: 'hello' } })
        .toJSON(),
    ).toEqual({ three: { p1: null, p2: 'hello' } });

    expect(program.registry.createType('GenericEnum<Tuple<u8>,Array<String>>', 'One').toJSON()).toEqual({
      one: null,
    });
    expect(program.registry.createType('GenericEnum<Tuple<u8>,Array<String>>', { Two: tupleU8 }).toJSON()).toEqual({
      two: tupleU8,
    });
    expect(
      program.registry
        .createType('GenericEnum<Tuple<u8>,Array<String>>', { Three: { p1: tupleU8Alt, p2: null } })
        .toJSON(),
    ).toEqual({ three: { p1: tupleU8Alt, p2: null } });

    expect(
      program.registry.createType('GenericEnum<Array<Tuple<u8>>,Tuple<Array<String>>>', 'One').toJSON(),
    ).toEqual({ one: null });
    expect(
      program.registry
        .createType('GenericEnum<Array<Tuple<u8>>,Tuple<Array<String>>>', { Two: arrayTupleU8 })
        .toJSON(),
    ).toEqual({ two: arrayTupleU8 });
    expect(
      program.registry
        .createType('GenericEnum<Array<Tuple<u8>>,Tuple<Array<String>>>', {
          Three: { p1: arrayTupleU8, p2: null },
        })
        .toJSON(),
    ).toEqual({ three: { p1: arrayTupleU8, p2: null } });
    expect(
      program.registry
        .createType('GenericEnum<Array<Tuple<u8>>,Tuple<Array<String>>>', {
          Three: { p1: arrayTupleU8, p2: [['a', 'b', 'c', 'd'], null] },
        })
        .toJSON(),
    ).toEqual({ three: { p1: arrayTupleU8, p2: [['a', 'b', 'c', 'd'], null] } });
  });
});

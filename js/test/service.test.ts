import { Sails } from '../lib';

let sails: Sails;

beforeAll(async () => {
  sails = await Sails.new();
});

describe('service', () => {
  test('simple service', () => {
    const idl = `service {
      DoThis : (a1: str) -> u8;
    }`;

    const result = sails.parseIdl(idl);

    expect(result.scaleCodecTypes).toEqual({});
    expect(Object.keys(result.functions)).toHaveLength(1);

    expect(result.functions).toHaveProperty('DoThis');
    expect(result.functions.DoThis.args).toHaveLength(1);
    expect(result.functions.DoThis.args[0].name).toEqual('a1');
    expect(result.functions.DoThis.args[0].type).toEqual('String');
    expect(result.functions.DoThis.returnType).toEqual('u8');
    expect(result.functions.DoThis.isQuery).toBeFalsy();

    const payload = result.functions.DoThis.encodePayload('hello');

    expect(result.registry.createType('(String, String)', payload).toJSON()).toEqual(['DoThis', 'hello']);
  });

  test('service with multiple methods', () => {
    const idl = `
    type SimpleStruct = struct {
      a: str,
      b: u32,
    };

    type ComplexEnum = enum {
      One,
      Two: u32,
      Three: opt vec u8,
  };

    service {
      DoThis : (a1: u32, a2: struct { str, opt u8 }) -> result (str, u8);
      DoThat : (a1: ComplexEnum) -> str;
      query GetThis : (a1: str) -> u8;
      query GetThat : (a1: SimpleStruct) -> str;
    }`;

    const result = sails.parseIdl(idl);

    expect(Object.keys(result.functions)).toHaveLength(4);

    expect(result.functions).toHaveProperty('DoThis');
    expect(result.functions).toHaveProperty('DoThat');
    expect(result.functions).toHaveProperty('GetThis');
    expect(result.functions).toHaveProperty('GetThat');

    expect(result.functions.DoThis.args).toHaveLength(2);
    expect(result.functions.DoThis.args[0].name).toEqual('a1');
    expect(result.functions.DoThis.args[0].type).toEqual('u32');
    expect(result.functions.DoThis.args[1].name).toEqual('a2');
    expect(result.functions.DoThis.args[1].type).toEqual('(String, Option<u8>)');
    expect(result.functions.DoThis.returnType).toEqual('Result<String, u8>');
    expect(result.functions.DoThis.isQuery).toBeFalsy();
    let payload = result.functions.DoThis.encodePayload(1, ['hello', 2]);
    expect(result.registry.createType('(String, u32, (String, Option<u8>))', payload).toJSON()).toEqual([
      'DoThis',
      1,
      ['hello', 2],
    ]);

    expect(result.functions.DoThat.args).toHaveLength(1);
    expect(result.functions.DoThat.args[0].name).toEqual('a1');
    expect(result.functions.DoThat.args[0].type).toEqual('ComplexEnum');
    expect(result.functions.DoThat.returnType).toEqual('String');
    expect(result.functions.DoThat.isQuery).toBeFalsy();
    payload = result.functions.DoThat.encodePayload({ Two: 2 });
    expect(result.registry.createType('(String, ComplexEnum)', payload).toJSON()).toEqual(['DoThat', { two: 2 }]);

    expect(result.functions.GetThis.args).toHaveLength(1);
    expect(result.functions.GetThis.args[0].name).toEqual('a1');
    expect(result.functions.GetThis.args[0].type).toEqual('String');
    expect(result.functions.GetThis.returnType).toEqual('u8');
    expect(result.functions.GetThis.isQuery).toBeTruthy();
    payload = result.functions.GetThis.encodePayload('hello');
    expect(result.registry.createType('(String, String)', payload).toJSON()).toEqual(['GetThis', 'hello']);

    expect(result.functions.GetThat.args).toHaveLength(1);
    expect(result.functions.GetThat.args[0].name).toEqual('a1');
    expect(result.functions.GetThat.args[0].type).toEqual('SimpleStruct');
    expect(result.functions.GetThat.returnType).toEqual('String');
    expect(result.functions.GetThat.isQuery).toBeTruthy();
    payload = result.functions.GetThat.encodePayload({ a: 'hello', b: 1234 });
    expect(result.registry.createType('(String, SimpleStruct)', payload).toJSON()).toEqual([
      'GetThat',
      { a: 'hello', b: 1234 },
    ]);
  });

  test('service with ctor', () => {
    const idl = `
    constructor {
      New : (p1: u32);
    };

    service {
      DoThis : (a1: str) -> u8;
    }`;

    const result = sails.parseIdl(idl);

    expect(Object.keys(result.functions)).toHaveLength(1);
    expect(Object.keys(result.ctors).includes('New')).toBeTruthy();
    expect(Object.keys(result.ctors.New.args)).toHaveLength(1);
    expect([...result.ctors.New.encodePayload(1)]).toEqual([12, 78, 101, 119, 1, 0, 0, 0]);
  });
});

import { Sails } from '../lib';
import { hexToU8a } from '@polkadot/util';

let sails: Sails;

beforeAll(async () => {
  sails = await Sails.new();
});

describe('service', () => {
  test('simple service', () => {
    const idl = `service TestService {
      DoThis : (a1: str) -> u8;
    }`;

    const result = sails.parseIdl(idl);

    expect(result.scaleCodecTypes).toEqual({});
    expect(Object.keys(result.services.TestService.functions)).toHaveLength(1);

    expect(result.services.TestService.functions).toHaveProperty('DoThis');
    expect(result.services.TestService.functions.DoThis.args).toHaveLength(1);
    expect(result.services.TestService.functions.DoThis.args[0].name).toEqual('a1');
    expect(result.services.TestService.functions.DoThis.args[0].type).toEqual('String');
    expect(result.services.TestService.functions.DoThis.returnType).toEqual('u8');
    expect(result.services.TestService.functions.DoThis.isQuery).toBeFalsy();

    const payload = result.services.TestService.functions.DoThis.encodePayload('hello');

    expect(result.registry.createType('(String, String, String)', payload).toJSON()).toEqual([
      'TestService',
      'DoThis',
      'hello',
    ]);
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

    service TestService {
      DoThis : (a1: u32, a2: struct { str, opt u8 }) -> result (str, u8);
      DoThat : (a1: ComplexEnum) -> str;
      query GetThis : (a1: str) -> u8;
      query GetThat : (a1: SimpleStruct) -> str;
    }`;

    const result = sails.parseIdl(idl);

    expect(Object.keys(result.services.TestService.functions)).toHaveLength(4);

    expect(result.services.TestService.functions).toHaveProperty('DoThis');
    expect(result.services.TestService.functions).toHaveProperty('DoThat');
    expect(result.services.TestService.functions).toHaveProperty('GetThis');
    expect(result.services.TestService.functions).toHaveProperty('GetThat');

    expect(result.services.TestService.functions.DoThis.args).toHaveLength(2);
    expect(result.services.TestService.functions.DoThis.args[0].name).toEqual('a1');
    expect(result.services.TestService.functions.DoThis.args[0].type).toEqual('u32');
    expect(result.services.TestService.functions.DoThis.args[1].name).toEqual('a2');
    expect(result.services.TestService.functions.DoThis.args[1].type).toEqual('(String, Option<u8>)');
    expect(result.services.TestService.functions.DoThis.returnType).toEqual('Result<String, u8>');
    expect(result.services.TestService.functions.DoThis.isQuery).toBeFalsy();
    let payload = result.services.TestService.functions.DoThis.encodePayload(1, ['hello', 2]);
    expect(result.registry.createType('(String, String, u32, (String, Option<u8>))', payload).toJSON()).toEqual([
      'TestService',
      'DoThis',
      1,
      ['hello', 2],
    ]);

    expect(result.services.TestService.functions.DoThat.args).toHaveLength(1);
    expect(result.services.TestService.functions.DoThat.args[0].name).toEqual('a1');
    expect(result.services.TestService.functions.DoThat.args[0].type).toEqual('ComplexEnum');
    expect(result.services.TestService.functions.DoThat.returnType).toEqual('String');
    expect(result.services.TestService.functions.DoThat.isQuery).toBeFalsy();
    payload = result.services.TestService.functions.DoThat.encodePayload({ Two: 2 });
    expect(result.registry.createType('(String, String, ComplexEnum)', payload).toJSON()).toEqual([
      'TestService',
      'DoThat',
      { two: 2 },
    ]);

    expect(result.services.TestService.functions.GetThis.args).toHaveLength(1);
    expect(result.services.TestService.functions.GetThis.args[0].name).toEqual('a1');
    expect(result.services.TestService.functions.GetThis.args[0].type).toEqual('String');
    expect(result.services.TestService.functions.GetThis.returnType).toEqual('u8');
    expect(result.services.TestService.functions.GetThis.isQuery).toBeTruthy();
    payload = result.services.TestService.functions.GetThis.encodePayload('hello');
    expect(result.registry.createType('(String, String, String)', payload).toJSON()).toEqual([
      'TestService',
      'GetThis',
      'hello',
    ]);

    expect(result.services.TestService.functions.GetThat.args).toHaveLength(1);
    expect(result.services.TestService.functions.GetThat.args[0].name).toEqual('a1');
    expect(result.services.TestService.functions.GetThat.args[0].type).toEqual('SimpleStruct');
    expect(result.services.TestService.functions.GetThat.returnType).toEqual('String');
    expect(result.services.TestService.functions.GetThat.isQuery).toBeTruthy();
    payload = result.services.TestService.functions.GetThat.encodePayload({ a: 'hello', b: 1234 });
    expect(result.registry.createType('(String, String, SimpleStruct)', payload).toJSON()).toEqual([
      'TestService',
      'GetThat',
      { a: 'hello', b: 1234 },
    ]);
  });

  test('service with ctor', () => {
    const idl = `
    constructor {
      New : (p1: u32);
    };

    service TestService {
      DoThis : (a1: str) -> u8;
    }`;

    const result = sails.parseIdl(idl);

    expect(Object.keys(result.services.TestService.functions)).toHaveLength(1);
    expect(Object.keys(result.ctors).includes('New')).toBeTruthy();
    expect(Object.keys(result.ctors.New.args)).toHaveLength(1);
    expect([...hexToU8a(result.ctors.New.encodePayload(1))]).toEqual([12, 78, 101, 119, 1, 0, 0, 0]);
  });

  test('service with events', () => {
    const idl = `
    service TestService {
      DoThis : (a1: str) -> u8;

      events {
        ThisDone;
        ThatDone: u32;
        SomethingHappened: struct { str, u32 };
      }
    }`;

    const result = sails.parseIdl(idl);

    expect(Object.keys(result.services.TestService.events)).toHaveLength(3);

    expect(result.services.TestService.events).toHaveProperty('ThisDone');
    expect(result.services.TestService.events).toHaveProperty('ThatDone');
    expect(result.services.TestService.events).toHaveProperty('SomethingHappened');

    expect(result.services.TestService.events.ThisDone.type).toBe('Null');
    expect(result.services.TestService.events.ThatDone.type).toBe('u32');
    expect(result.services.TestService.events.SomethingHappened.type).toBe('(String, u32)');
  });
});

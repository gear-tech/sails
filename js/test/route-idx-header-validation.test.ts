import { SailsIdlParser, InterfaceId, SailsMessageHeader } from 'sails-js-parser-idl-v2';
import { u8aConcat, u8aToHex } from '@polkadot/util';

import { SailsProgram } from '..';

const SERVICE_INTERFACE_ID = '0x1dfeda616911b428';

const idl = `
  service Echo@${SERVICE_INTERFACE_ID} {
    functions {
      @entry_id: 1
      Ping(value: u32) -> u32;
    }
    events {
      @entry_id: 2
      Pinged(u32),
    }
  }

  program EchoProgram {
    constructors {
      @entry_id: 0
      New();
    }
    services {
      Echo
    }
  }
`;

let program: SailsProgram;

beforeAll(async () => {
  const parser = new SailsIdlParser();
  await parser.init();
  program = new SailsProgram(parser.parse(idl));
});

describe('_assertMatchingHeader route_idx validation', () => {
  test('parser auto-assigns route_idx = 1 for the first mounted service', () => {
    expect(program.services.Echo.routeIdx).toBe(1);
  });

  test('function decodePayload throws when received route_idx differs', () => {
    const service = program.services.Echo;

    const correctPayload = service.functions.Ping.encodePayload(42);
    expect(service.functions.Ping.decodePayload(correctPayload)).toEqual({ value: 42 });

    const wrongHeader = SailsMessageHeader.v1(InterfaceId.from(SERVICE_INTERFACE_ID), 1, 99);
    const body = service.registry.createType('u32', 42).toU8a();
    const wrongPayload = u8aToHex(u8aConcat(wrongHeader.toBytes(), body));

    expect(() => service.functions.Ping.decodePayload(wrongPayload)).toThrow(
      /Header mismatch.*route_idx=1.*route_idx=99/s,
    );
  });

  test('event decode throws when received route_idx differs', () => {
    const service = program.services.Echo;

    const wrongHeader = SailsMessageHeader.v1(InterfaceId.from(SERVICE_INTERFACE_ID), 2, 99);
    const body = service.registry.createType('u32', 7).toU8a();
    const wrongPayload = u8aToHex(u8aConcat(wrongHeader.toBytes(), body));

    expect(() => service.events.Pinged.decode(wrongPayload)).toThrow(
      /Header mismatch.*route_idx=1.*route_idx=99/s,
    );
  });

  test('matching route_idx decodes successfully (function + event)', () => {
    const service = program.services.Echo;

    const funcPayload = service.functions.Ping.encodePayload(42);
    expect(service.functions.Ping.decodePayload(funcPayload)).toEqual({ value: 42 });

    const eventHeader = SailsMessageHeader.v1(InterfaceId.from(SERVICE_INTERFACE_ID), 2, service.routeIdx);
    const eventBody = service.registry.createType('u32', 7).toU8a();
    const eventPayload = u8aToHex(u8aConcat(eventHeader.toBytes(), eventBody));
    expect(service.events.Pinged.decode(eventPayload)).toEqual(7);
  });

  test('ctor decodePayload accepts any received route_idx (asymmetric expected-only rule)', () => {
    const headerWithRoute = SailsMessageHeader.v1(InterfaceId.zero(), 0, 5);
    const payload = u8aToHex(headerWithRoute.toBytes());

    expect(() => program.ctors.New.decodePayload(payload)).not.toThrow();
  });
});

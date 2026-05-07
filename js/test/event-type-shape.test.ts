import { SailsIdlParser, InterfaceId, SailsMessageHeader } from 'sails-js-parser-idl-v2';
import { u8aConcat, u8aToHex } from '@polkadot/util';

import { SailsProgram } from '..';

const SERVICE_INTERFACE_ID = '0xafbdac53c6ba06b7';

const idl = `
  service EventShapes@${SERVICE_INTERFACE_ID} {
    events {
      Bare,
      WithU32(u32),
      Structured {
        from: (i32, i32),
        to: (i32, i32),
      },
    }
    functions {
      @query
      Noop() -> bool;
    }
  }

  program EventShapesProgram {
    services {
      EventShapes
    }
  }
`;

let program: SailsProgram;

beforeAll(async () => {
  const parser = new SailsIdlParser();
  await parser.init();
  program = new SailsProgram(parser.parse(idl));
});

describe('events[x].type uniform string shape', () => {
  test('unit variant returns string "Null"', () => {
    const event = program.services.EventShapes.events.Bare;
    expect(typeof event.type).toBe('string');
    expect(event.type).toBe('Null');
  });

  test('tuple/unnamed variant returns string', () => {
    const event = program.services.EventShapes.events.WithU32;
    expect(typeof event.type).toBe('string');
  });

  test('named-field variant returns string (KEY ASSERTION)', () => {
    const event = program.services.EventShapes.events.Structured;
    expect(typeof event.type).toBe('string');
  });

  test('round-trip: encoded named-field event decodes to original payload', () => {
    const service = program.services.EventShapes;
    const event = service.events.Structured;

    const interfaceId = InterfaceId.from(SERVICE_INTERFACE_ID);
    const entryId = event.typeDef.entry_id ?? 0;
    const header = SailsMessageHeader.v1(interfaceId, entryId, service.routeIdx);

    const value = { from: [1, 2], to: [3, 4] };
    const body = service.registry.createType(event.type, value).toU8a();
    const merged = u8aConcat(header.toBytes(), body);

    expect(event.decode(u8aToHex(merged))).toEqual(value);
  });

  test('event.typeDef.fields escape hatch is preserved for named-field variants', () => {
    const event = program.services.EventShapes.events.Structured;
    expect(event.typeDef.fields).toBeDefined();
    expect(event.typeDef.fields?.length).toBe(2);
    expect(event.typeDef.fields?.map((f) => f.name)).toEqual(['from', 'to']);
  });
});

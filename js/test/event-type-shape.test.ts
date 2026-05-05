import { SailsIdlParser, InterfaceId, SailsMessageHeader } from 'sails-js-parser-idl-v2';
import { u8aToHex } from '@polkadot/util';

import { SailsProgram } from '..';

let parser: SailsIdlParser;

beforeAll(async () => {
  parser = new SailsIdlParser();
  await parser.init();
});

const idl = `
  service EventShapes@0xafbdac53c6ba06b7 {
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

describe('events[x].type uniform string shape', () => {
  test('unit variant returns string "Null"', () => {
    const program = new SailsProgram(parser.parse(idl));
    const event = program.services.EventShapes.events.Bare;
    expect(typeof event.type).toBe('string');
    expect(event.type).toBe('Null');
  });

  test('tuple/unnamed variant returns string', () => {
    const program = new SailsProgram(parser.parse(idl));
    const event = program.services.EventShapes.events.WithU32;
    expect(typeof event.type).toBe('string');
  });

  test('named-field variant returns string (KEY ASSERTION — was an object before this fix)', () => {
    const program = new SailsProgram(parser.parse(idl));
    const event = program.services.EventShapes.events.Structured;
    expect(typeof event.type).toBe('string');
  });

  test('round-trip: encoded named-field event decodes to original payload', () => {
    const program = new SailsProgram(parser.parse(idl));
    const service = program.services.EventShapes;
    const event = service.events.Structured;

    const interfaceId = InterfaceId.from('0xafbdac53c6ba06b7');
    const entryId = (event.typeDef as any).entry_id ?? 0;
    const header = SailsMessageHeader.v1(interfaceId, entryId, service.routeIdx);

    const value = { from: [1, 2], to: [3, 4] };
    const body = service.registry.createType(event.type, value).toU8a();

    const merged = new Uint8Array(header.toBytes().length + body.length);
    merged.set(header.toBytes(), 0);
    merged.set(body, header.toBytes().length);

    expect(event.decode(u8aToHex(merged))).toEqual(value);
  });

  test('event.typeDef.fields escape hatch is preserved for named-field variants', () => {
    const program = new SailsProgram(parser.parse(idl));
    const event = program.services.EventShapes.events.Structured;
    expect(event.typeDef.fields).toBeDefined();
    expect(event.typeDef.fields?.length).toBe(2);
    expect(event.typeDef.fields?.map((f) => f.name)).toEqual(['from', 'to']);
  });
});

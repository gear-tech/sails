import { SailsIdlParser, InterfaceId, SailsMessageHeader, normalizeIdl } from 'sails-js-parser-idl-v2';
import { hexToU8a } from '@polkadot/util';

import { SailsProgram, ZERO_ADDRESS } from '..';

let parser: SailsIdlParser;

beforeAll(async () => {
  parser = new SailsIdlParser();
  await parser.init();
});

describe('partial service and entry-id', () => {
  test('resolves entry_id from annotation for function headers', () => {
    const idl = `
      @partial
      service PartialService@0x1234567890abcdef {
        functions {
          @entry-id: 0
          AMethodDefaultId0() -> bool;
          @entry-id: 5
          BMethodWithId5() -> bool;
          @entry-id: 2
          CMethodDefaultId2() -> u32;
        }
      }

      program PartialProgram {
        services {
          PartialService
        }
      }
    `;

    const program = new SailsProgram(parser.parse(idl));
    const service = program.services.PartialService;

    const payloadFirst = service.functions.AMethodDefaultId0.encodePayload();
    const { header: headerFirst } = SailsMessageHeader.tryReadBytes(hexToU8a(payloadFirst));
    expect(headerFirst.entryId).toBe(0);

    const payloadSecond = service.functions.BMethodWithId5.encodePayload();
    const { header: headerSecond } = SailsMessageHeader.tryReadBytes(hexToU8a(payloadSecond));
    expect(headerSecond.entryId).toBe(5);
    expect(headerSecond.interfaceId.toString()).toBe('0x1234567890abcdef');

    const payloadThird = service.functions.CMethodDefaultId2.encodePayload();
    const { header: headerThird } = SailsMessageHeader.tryReadBytes(hexToU8a(payloadThird));
    expect(headerThird.entryId).toBe(2);
  });

  test('resolves entry_id from annotation for events', () => {
    const idl = `
      @partial
      service PartialService@0x1234567890abcdef {
        events {
          @entry-id: 10
          EventWithId10(String);
        }
      }

      program PartialProgram {
        services {
          PartialService
        }
      }
    `;

    const program = new SailsProgram(parser.parse(idl));
    const service = program.services.PartialService;

    const interfaceId = InterfaceId.from('0x1234567890abcdef');
    const header = SailsMessageHeader.v1(interfaceId, 10, 0);

    const isMatch = service.events.EventWithId10.is({
      data: {
        message: {
          destination: { eq: (addr: string) => addr === ZERO_ADDRESS },
          payload: header.toBytes(),
        },
      },
    } as any);

    expect(isMatch).toBe(true);
  });

  test('falls back to index when legacy normalized docs omit entry_id', () => {
    const doc = normalizeIdl({
      program: {
        name: 'LegacyProgram',
        ctors: [
          { name: 'Default', params: [] },
          { name: 'Create', params: [], annotations: [['entry-id', '7']] },
        ],
        services: [
          {
            name: 'LegacyService',
            interface_id: '0x1234567890abcdef',
            route_idx: 1,
          },
        ],
        types: [],
      },
      services: [
        {
          name: 'LegacyService',
          interface_id: '0x1234567890abcdef',
          funcs: [
            { name: 'DefaultCall', params: [], output: 'bool', kind: 'command' },
            {
              name: 'AnnotatedCall',
              params: [],
              output: 'bool',
              kind: 'command',
              annotations: [['entry-id', '5']],
            },
          ],
          events: [
            { name: 'DefaultEvent', fields: [] },
            { name: 'AnnotatedEvent', fields: [], annotations: [['entry-id', '10']] },
          ],
          types: [],
        },
      ],
    });

    expect(doc.program?.ctors?.map((ctor) => ctor.entry_id)).toEqual([0, 1]);
    expect(doc.services?.[0]?.funcs?.map((func) => func.entry_id)).toEqual([0, 1]);
    expect(doc.services?.[0]?.events?.map((event) => event.entry_id)).toEqual([0, 1]);

    const program = new SailsProgram(doc);

    const ctorPayload = program.ctors.Create.encodePayload();
    const { header: ctorHeader } = SailsMessageHeader.tryReadBytes(hexToU8a(ctorPayload));
    expect(ctorHeader.entryId).toBe(1);

    const servicePayload = program.services.LegacyService.functions.AnnotatedCall.encodePayload();
    const { header: serviceHeader } = SailsMessageHeader.tryReadBytes(hexToU8a(servicePayload));
    expect(serviceHeader.entryId).toBe(1);

    const interfaceId = InterfaceId.from('0x1234567890abcdef');
    const eventHeader = SailsMessageHeader.v1(interfaceId, 1, 0);
    const isMatch = program.services.LegacyService.events.AnnotatedEvent.is({
      data: {
        message: {
          destination: { eq: (addr: string) => addr === ZERO_ADDRESS },
          payload: eventHeader.toBytes(),
        },
      },
    } as any);

    expect(isMatch).toBe(true);
  });
});

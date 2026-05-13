import { SailsIdlParser, InterfaceId, SailsMessageHeader } from 'sails-js-parser-idl-v2';

import { SailsProgram } from '../src/sails-idl-v2';

const idl = `
  service Counter {
    functions {
      @entry_id: 1
      Add(value: u32) -> u32 throws String;

      @query
      @entry_id: 2
      Get() -> u32;

      @entry_id: 4
      SetName(name: String);
    }

    events {
      @entry_id: 3
      Added(u32),
    }
  }

  program CounterProgram {
    constructors {
      @entry_id: 0
      New(seed: u32);
    }

    services {
      Counter: Primary,
      Counter: Secondary,
    }
  }
`;

const singleRouteIdl = `
  service Counter {
    functions {
      @entry_id: 1
      Add(value: u32) -> u32 throws String;

      @query
      @entry_id: 2
      Get() -> u32;

      @entry_id: 4
      SetName(name: String);
    }
  }

  program CounterProgram {
    constructors {
      @entry_id: 0
      New(seed: u32);
    }

    services {
      Counter
    }
  }
`;

let parser: SailsIdlParser;

const parseProgram = (text: string): SailsProgram => new SailsProgram(parser.parse(text));
const decoder = (program: SailsProgram): any => program;
const counterIid = (program: SailsProgram): InterfaceId => InterfaceId.from((program as any)._doc.services[0].interface_id);

const body = (program: SailsProgram, type: string, value: unknown): Uint8Array =>
  new Uint8Array(program.registry.createType<any>(type, value).toU8a()) as Uint8Array<ArrayBuffer>;

const concat = (left: Uint8Array, right: Uint8Array): any => {
  const out = new Uint8Array(left.length + right.length);
  out.set(left, 0);
  out.set(right, left.length);
  return out;
};

const payload = (header: SailsMessageHeader, bytes: any = new Uint8Array()): any => concat(header.toBytes(), bytes);

beforeAll(async () => {
  parser = new SailsIdlParser();
  await parser.init();
});

describe('SailsProgram dispatcher API', () => {
  test('resolves route_idx-specific entries and returns candidates for ambiguous inference', () => {
    const program = parseProgram(idl);
    const iid = counterIid(program);
    const header = SailsMessageHeader.v1(iid, 1, 2);

    expect(program.resolveEntry(header)).toMatchObject({
      kind: 'command',
      service: 'Counter',
      fn: 'Add',
      route: 'Secondary',
      entryId: 1,
      route_idx: 2,
    });

    expect(program.resolveEntry(SailsMessageHeader.v1(iid, 1, 0))).toMatchObject({
      kind: 'unknown',
      reason: 'ambiguous-route',
    });
    expect(program.resolveEntryCandidates(iid).map((entry) => entry.route_idx)).toEqual([
      1, 1, 1, 1, 2, 2, 2, 2,
    ]);
  });

  test('decodes calls, replies, errors, events, and constructors', () => {
    const program = parseProgram(idl);
    const iid = counterIid(program);
    const addHeader = SailsMessageHeader.v1(iid, 1, 2);
    const eventHeader = SailsMessageHeader.v1(iid, 3, 2);
    const ctorHeader = SailsMessageHeader.v1(InterfaceId.zero(), 0, 0);

    const call = decoder(program).decodeCall(payload(addHeader, body(program, 'u32', 7)));
    expect(call).toMatchObject({ kind: 'call', args: { value: 7 } });
    if (call.kind !== 'unknown') {
      expect(decoder(program).decodeReply(payload(addHeader, body(program, 'u32', 11)), call.entry)).toMatchObject({
        kind: 'reply',
        result: 11,
      });
      expect(
        decoder(program).decodeReply(payload(addHeader, body(program, 'u32', 12)), {
          ...call.entry,
          route_idx: 0,
        }),
      ).toMatchObject({
        kind: 'reply',
        result: 12,
      });
      expect(decoder(program).decodeError(payload(addHeader, body(program, 'String', 'boom')), call.entry)).toMatchObject({
        kind: 'error',
        error: 'boom',
      });
    }

    expect(decoder(program).decodeEvent(payload(eventHeader, body(program, 'u32', 9)))).toMatchObject({
      kind: 'event',
      data: 9,
    });
    expect(decoder(program).decodeCtor(payload(ctorHeader, body(program, 'u32', 42)))).toMatchObject({
      kind: 'ctor-call',
      args: { seed: 42 },
    });
  });

  test('returns typed unknown results for invalid or mismatched bytes', () => {
    const program = parseProgram(singleRouteIdl);
    const iid = counterIid(program);
    const addHeader = SailsMessageHeader.v1(iid, 1, 1);
    const eventHeader = SailsMessageHeader.v1(iid, 3, 1);
    const getHeader = SailsMessageHeader.v1(iid, 2, 1);
    const setNameHeader = SailsMessageHeader.v1(iid, 4, 1);

    expect(decoder(program).decodeCall(new Uint8Array())).toMatchObject({ kind: 'unknown', reason: 'too-short' });
    expect(decoder(program).decodeCall(new Uint8Array(15))).toMatchObject({ kind: 'unknown', reason: 'too-short' });

    const badMagic = payload(addHeader, body(program, 'u32', 7));
    badMagic[0] = 0;
    expect(decoder(program).decodeCall(badMagic)).toMatchObject({ kind: 'unknown', reason: 'no-magic' });

    const badVersion = payload(addHeader, body(program, 'u32', 7));
    badVersion[2] = 2;
    expect(decoder(program).decodeCall(badVersion)).toMatchObject({ kind: 'unknown', reason: 'bad-version' });

    const badReserved = payload(addHeader, body(program, 'u32', 7));
    badReserved[15] = 1;
    expect(decoder(program).decodeCall(badReserved)).toMatchObject({ kind: 'unknown', reason: 'bad-reserved' });

    const badHlen = payload(addHeader, body(program, 'u32', 7));
    badHlen[3] = 17;
    expect(decoder(program).decodeCall(badHlen)).toMatchObject({ kind: 'unknown', reason: 'bad-hlen' });

    expect(
      decoder(program).decodeCall(payload(SailsMessageHeader.v1(InterfaceId.fromU64(0x1234n), 1, 1), body(program, 'u32', 7))),
    ).toMatchObject({ kind: 'unknown', reason: 'no-service' });
    expect(decoder(program).decodeCall(payload(eventHeader))).toMatchObject({ kind: 'unknown', reason: 'no-entry' });
    expect(decoder(program).decodeEvent(payload(addHeader, body(program, 'u32', 7)))).toMatchObject({
      kind: 'unknown',
      reason: 'entry-mismatch',
    });
    expect(decoder(program).decodeCall(payload(setNameHeader, new Uint8Array([0xFF])))).toMatchObject({
      kind: 'unknown',
      reason: 'decode-failed',
    });
    expect(
      decoder(program).decodeReply(payload(addHeader, body(program, 'u32', 7)), {
        kind: 'command',
        service: 'Counter',
        fn: 'Add',
        route: 'Counter',
        interfaceId: iid,
        entryId: 1,
        route_idx: 99,
      }),
    ).toMatchObject({ kind: 'unknown', reason: 'entry-mismatch' });
    expect(decoder(program).decodeError(payload(getHeader, body(program, 'u32', 7)))).toMatchObject({
      kind: 'unknown',
      reason: 'no-throws-type',
    });
    expect(decoder(program).decodeCall(concat(payload(addHeader, body(program, 'u32', 7)), new Uint8Array([1])))).toMatchObject({
      kind: 'unknown',
      reason: 'trailing-bytes',
      consumedLen: 4,
    });
  });

  test('route_idx zero resolves only when a single route matches', () => {
    const program = parseProgram(singleRouteIdl);
    const header = SailsMessageHeader.v1(counterIid(program), 1, 0);

    expect(decoder(program).decodeCall(payload(header, body(program, 'u32', 7)))).toMatchObject({
      kind: 'call',
      args: { value: 7 },
    });
  });

  test('random short inputs never throw from dispatcher methods', () => {
    const program = parseProgram(singleRouteIdl);

    for (let len = 0; len <= 64; len += 1) {
      const bytes = Uint8Array.from({ length: len }, (_, i) => (i * 31 + len) & 0xFF);
      expect(() => decoder(program).decodeCall(bytes)).not.toThrow();
      expect(() => decoder(program).decodeReply(bytes)).not.toThrow();
      expect(() => decoder(program).decodeError(bytes)).not.toThrow();
      expect(() => decoder(program).decodeEvent(bytes)).not.toThrow();
      expect(() => decoder(program).decodeCtor(bytes)).not.toThrow();
    }
  });
});

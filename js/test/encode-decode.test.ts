import { readFileSync } from 'node:fs';
import { getCtorNamePrefix, getFnNamePrefix as getFunctionNamePrefix, getServiceNamePrefix, Sails } from '..';
import { SailsIdlParser } from 'sails-js-parser';

const DEMO_IDL_PATH = 'test/demo/demo.idl';

let sails: Sails;
const demoIdl = readFileSync(DEMO_IDL_PATH, 'utf8');

beforeAll(async () => {
  const parser = await SailsIdlParser.new();
  sails = new Sails(parser);
  sails.parseIdl(demoIdl);
});

describe('Encode/Decode', () => {
  test('encode/decode demo ctors', async () => {
    const newEncoded = sails.ctors.New.encodePayload(null, [5, 5]);

    expect(newEncoded).toBe('0x0c4e657700010500000005000000');

    const newDecoded = sails.ctors.New.decodePayload(newEncoded);

    expect(newDecoded).toEqual({ counter: null, dog_position: [5, 5] });

    expect(getCtorNamePrefix(newEncoded)).toBe('New');
  });

  test('encode/decode dog walk', async () => {
    const walkEncoded = sails.services.Dog.functions.Walk.encodePayload(10, 10);

    expect(walkEncoded).toBe('0x0c446f671057616c6b0a0000000a000000');

    const walkDecoded = sails.services.Dog.functions.Walk.decodePayload(walkEncoded);

    expect(walkDecoded).toEqual({ dx: 10, dy: 10 });

    expect(getServiceNamePrefix(walkEncoded)).toBe('Dog');
    expect(getFunctionNamePrefix(walkEncoded)).toBe('Walk');
  });

  test('decodeResult validates service/function prefix', () => {
    const add = sails.services.Counter.functions.Add;
    const validReply = sails.registry.createType('(String, String, u32)', ['Counter', 'Add', 99]).toHex();
    expect(add.decodeResult(validReply)).toBe(99);

    const mismatchedReply = sails.registry
      .createType('(String, String, u32)', ['WrongService', 'Add', 99])
      .toHex();
    expect(() => add.decodeResult(mismatchedReply)).toThrow('Invalid prefix for Counter.Add result');
  });

  test('decodeResult throws a clear error on truncated reply bytes', () => {
    const add = sails.services.Counter.functions.Add;
    // Empty bytes / too short to even contain the compact-prefixed service name —
    // the prefix helpers throw from the SCALE codec; decodeResult should surface
    // a single, consistent "Invalid prefix" error instead.
    expect(() => add.decodeResult('0x')).toThrow(/Invalid prefix for Counter\.Add result/);
    expect(() => add.decodeResult('0x01')).toThrow(/Invalid prefix for Counter\.Add result/);
  });

  test('decodePayload validates service/function prefix', () => {
    const add = sails.services.Counter.functions.Add;
    const mismatched = sails.registry.createType('(String, String)', ['WrongService', 'Add']).toHex();
    expect(() => add.decodePayload(mismatched)).toThrow(/Invalid prefix for Counter\.Add/);
    expect(() => add.decodePayload('0x')).toThrow(/Invalid prefix for Counter\.Add/);
  });

  test('ctor decodePayload validates constructor prefix', () => {
    const { New } = sails.ctors;
    const mismatched = sails.registry.createType('String', 'Default').toHex();
    expect(() => New.decodePayload(mismatched)).toThrow(/Invalid prefix for constructor "New"/);
    expect(() => New.decodePayload('0x')).toThrow(/Invalid prefix for constructor "New"/);
  });

  test('event decode validates service/event prefix', () => {
    const added = sails.services.Counter.events.Added;
    const mismatched = sails.registry.createType('(String, String)', ['Counter', 'WrongEvent']).toHex();
    expect(() => added.decode(mismatched)).toThrow(/Invalid prefix for Counter\.Added/);
  });

  test('decodePayload accepts Uint8Array inputs (back-compat with as any callers)', () => {
    const walk = sails.services.Dog.functions.Walk;
    // Round-trip via Uint8Array rather than HexString — this flow was valid before
    // the prefix guards were added (registry.createType accepts Uint8Array).
    const walkHex = walk.encodePayload(10, 10);
    const walkBytes = Buffer.from(walkHex.slice(2), 'hex');
    const decoded = walk.decodePayload(walkBytes as unknown as `0x${string}`);
    expect(decoded).toEqual({ dx: 10, dy: 10 });
  });

  test('error messages sanitize non-printable prefix content', () => {
    const add = sails.services.Counter.functions.Add;
    // Craft a prefix with embedded control characters (ESC + BEL). Our error
    // should replace them with "?" rather than pipe them into logs verbatim.
    const esc = String.fromCodePoint(0x1B);
    const bel = String.fromCodePoint(0x07);
    const mismatched = sails.registry
      .createType('(String, String)', [`Counter${esc}[31m`, `Add${bel}`])
      .toHex();
    try {
      add.decodePayload(mismatched);
      throw new Error('expected decodePayload to throw');
    } catch (error) {
      const msg = (error as Error).message;
      expect(msg).toContain('Invalid prefix for Counter.Add');
      expect(msg).not.toContain(esc);
      expect(msg).not.toContain(bel);
    }
  });
});

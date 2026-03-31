import { describe, test, expect, beforeAll } from 'bun:test';
import { SailsIdlParser } from 'sails-js-parser-idl-v2';
import { registry } from '../src/registry';
import { readFile } from 'node:fs/promises';
import { resolve, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';

const thisDir = dirname(fileURLToPath(import.meta.url));
const repoRoot = resolve(thisDir, '..', '..', '..');
const demoIdlPath = resolve(repoRoot, 'js/test/demo-v2/demo.idl');

beforeAll(async () => {
  const parser = new SailsIdlParser();
  await parser.init();
  const idl = await readFile(demoIdlPath, 'utf-8');
  const doc = parser.parse(idl);
  registry.register('DemoClient', doc);
});

describe('SCALE codec encoding/decoding', () => {
  test('encode Counter.Add(42)', () => {
    const entry = registry.getOrThrow('DemoClient');
    const counter = entry.program.services['Counter'];
    expect(counter).toBeDefined();

    const addFn = counter.functions['Add'];
    expect(addFn).toBeDefined();

    const hex = addFn.encodePayload(42);
    expect(hex).toBeDefined();
    expect(hex.startsWith('0x')).toBe(true);

    // Header is 16 bytes = 32 hex chars + "0x" prefix
    // u32 is 4 bytes = 8 hex chars
    // Total: 2 + 32 + 8 = 42 chars
    expect(hex.length).toBe(42);

    // First two bytes should be "GM" magic (0x474d)
    expect(hex.slice(2, 6)).toBe('474d');
  });

  test('encode then decode Counter.Add round-trip', () => {
    const entry = registry.getOrThrow('DemoClient');
    const addFn = entry.program.services['Counter'].functions['Add'];

    const hex = addFn.encodePayload(42);
    const decoded = addFn.decodePayload(hex);

    expect(decoded).toEqual({ value: 42 });
  });

  test('encode then decode result round-trip', () => {
    const entry = registry.getOrThrow('DemoClient');
    const addFn = entry.program.services['Counter'].functions['Add'];

    // Encode a payload, then decode it as a result
    // The result format is the same: [u8;16] header + return_type
    const hex = addFn.encodePayload(99);
    const result = addFn.decodeResult(hex);

    // The result is decoded as u32, should be 99
    expect(result).toBe(99);
  });

  test('encode PingPong.Ping with string arg', () => {
    const entry = registry.getOrThrow('DemoClient');
    const pingFn = entry.program.services['PingPong'].functions['Ping'];
    expect(pingFn).toBeDefined();

    const hex = pingFn.encodePayload('hello');
    expect(hex.startsWith('0x474d')).toBe(true);

    const decoded = pingFn.decodePayload(hex);
    expect(decoded).toEqual({ input: 'hello' });
  });

  test('encode constructor Default()', () => {
    const entry = registry.getOrThrow('DemoClient');
    const ctors = entry.program.ctors;
    expect(ctors).toBeDefined();
    expect(ctors!['Default']).toBeDefined();

    const hex = ctors!['Default'].encodePayload();
    expect(hex.startsWith('0x')).toBe(true);
    // Default() takes no args, should be just the 16-byte header
    expect(hex.length).toBe(34); // 0x + 32 hex chars
  });

  test('encode constructor New with args', () => {
    const entry = registry.getOrThrow('DemoClient');
    const ctor = entry.program.ctors!['New'];
    expect(ctor).toBeDefined();

    const hex = ctor.encodePayload(42, [10, 20]);
    expect(hex.startsWith('0x')).toBe(true);

    const decoded = ctor.decodePayload(hex);
    expect(decoded.counter).toBe(42);
    expect(decoded.dog_position).toEqual([10, 20]);
  });

  test('function with no args (Counter.Value query)', () => {
    const entry = registry.getOrThrow('DemoClient');
    const valueFn = entry.program.services['Counter'].queries['Value'];
    expect(valueFn).toBeDefined();

    const hex = valueFn.encodePayload();
    expect(hex.startsWith('0x474d')).toBe(true);
    // No args = just 16-byte header
    expect(hex.length).toBe(34);
  });
});

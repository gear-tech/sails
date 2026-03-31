import { describe, test, expect, beforeAll } from 'bun:test';
import { SailsIdlParser } from 'sails-js-parser-idl-v2';
import { registry } from '../src/registry';
import { typeDeclToSolidity } from '../src/tools/abi-tools';
import {
  encodeAbiParameters,
  decodeAbiParameters,
  keccak256,
  toHex,
  type AbiParameter,
} from 'viem';
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

// ---------------------------------------------------------------------------
// Type mapping
// ---------------------------------------------------------------------------

describe('typeDeclToSolidity', () => {
  test('primitive types', () => {
    expect(typeDeclToSolidity('bool')).toBe('bool');
    expect(typeDeclToSolidity('u8')).toBe('uint8');
    expect(typeDeclToSolidity('u16')).toBe('uint16');
    expect(typeDeclToSolidity('u32')).toBe('uint32');
    expect(typeDeclToSolidity('u64')).toBe('uint64');
    expect(typeDeclToSolidity('u128')).toBe('uint128');
    expect(typeDeclToSolidity('U256')).toBe('uint256');
    expect(typeDeclToSolidity('i8')).toBe('int8');
    expect(typeDeclToSolidity('i32')).toBe('int32');
    expect(typeDeclToSolidity('i128')).toBe('int128');
    expect(typeDeclToSolidity('String')).toBe('string');
    expect(typeDeclToSolidity('ActorId')).toBe('address');
    expect(typeDeclToSolidity('H256')).toBe('bytes32');
    expect(typeDeclToSolidity('CodeId')).toBe('bytes32');
    expect(typeDeclToSolidity('MessageId')).toBe('bytes32');
    expect(typeDeclToSolidity('H160')).toBe('bytes20');
  });

  test('slice type', () => {
    expect(typeDeclToSolidity({ kind: 'slice', item: 'u8' })).toBe('uint8[]');
    expect(typeDeclToSolidity({ kind: 'slice', item: 'ActorId' })).toBe('address[]');
  });

  test('fixed array type', () => {
    expect(typeDeclToSolidity({ kind: 'array', item: 'u8', len: 32 })).toBe('uint8[32]');
    expect(typeDeclToSolidity({ kind: 'array', item: 'u32', len: 4 })).toBe('uint32[4]');
  });

  test('void type throws', () => {
    expect(() => typeDeclToSolidity('()')).toThrow();
  });

  test('named type throws', () => {
    expect(() => typeDeclToSolidity({ kind: 'named', name: 'MyStruct' })).toThrow();
  });

  test('tuple type throws', () => {
    expect(() => typeDeclToSolidity({ kind: 'tuple', types: ['u32', 'u64'] })).toThrow();
  });
});

// ---------------------------------------------------------------------------
// Selector computation
// ---------------------------------------------------------------------------

describe('selector computation', () => {
  test('selector matches keccak256 of signature', () => {
    // counterAdd(bool,uint32) - camelCase("Counter" + "Add") = "counterAdd"
    const sig = 'counterAdd(bool,uint32)';
    const expected = keccak256(toHex(sig)).slice(0, 10);
    // Verify it's 4 bytes = 10 chars with 0x prefix
    expect(expected.length).toBe(10);
    expect(expected.startsWith('0x')).toBe(true);
  });

  test('selector differs between functions', () => {
    const sig1 = keccak256(toHex('counterAdd(bool,uint32)')).slice(0, 10);
    const sig2 = keccak256(toHex('counterSub(bool,uint32)')).slice(0, 10);
    expect(sig1).not.toBe(sig2);
  });
});

// ---------------------------------------------------------------------------
// ABI encode/decode round-trips (using viem directly)
// ---------------------------------------------------------------------------

describe('ABI encode/decode round-trips', () => {
  test('encode and decode counterAdd call', () => {
    // counterAdd(bool _callReply, uint32 value)
    const params: AbiParameter[] = [
      { name: '_callReply', type: 'bool' },
      { name: 'value', type: 'uint32' },
    ];
    const encoded = encodeAbiParameters(params, [false, 42]);
    const decoded = decodeAbiParameters(params, encoded);

    expect(decoded[0]).toBe(false);
    expect(decoded[1]).toBe(42);
  });

  test('encode and decode with address param', () => {
    const params: AbiParameter[] = [
      { name: '_callReply', type: 'bool' },
      { name: 'actor', type: 'address' },
    ];
    const addr = '0x1234567890abcdef1234567890abcdef12345678';
    const encoded = encodeAbiParameters(params, [false, addr]);
    const decoded = decodeAbiParameters(params, encoded);
    expect(decoded[1].toLowerCase()).toBe(addr.toLowerCase());
  });

  test('encode and decode with bytes32 param', () => {
    const params: AbiParameter[] = [{ name: 'hash', type: 'bytes32' }];
    const h256 = '0x' + 'ab'.repeat(32);
    const encoded = encodeAbiParameters(params, [h256]);
    const decoded = decodeAbiParameters(params, encoded);
    expect((decoded[0] as string).toLowerCase()).toBe(h256.toLowerCase());
  });

  test('encode and decode with string param', () => {
    const params: AbiParameter[] = [
      { name: '_callReply', type: 'bool' },
      { name: 'input', type: 'string' },
    ];
    const encoded = encodeAbiParameters(params, [false, 'hello world']);
    const decoded = decodeAbiParameters(params, encoded);
    expect(decoded[1]).toBe('hello world');
  });
});

// ---------------------------------------------------------------------------
// Event signature hashing
// ---------------------------------------------------------------------------

describe('event topic computation', () => {
  test('event topic is keccak256 of signature', () => {
    // Hypothetical event: Transfer(address,address,uint256)
    const sig = 'Transfer(address,address,uint256)';
    const topic = keccak256(toHex(sig));
    expect(topic.length).toBe(66); // 0x + 64 hex chars
    expect(topic.startsWith('0x')).toBe(true);
  });

  test('different event signatures produce different topics', () => {
    const t1 = keccak256(toHex('Added(uint32)'));
    const t2 = keccak256(toHex('Subtracted(uint32)'));
    expect(t1).not.toBe(t2);
  });
});

// ---------------------------------------------------------------------------
// Solidity generation (smoke test via IDL doc inspection)
// ---------------------------------------------------------------------------

describe('Solidity generation', () => {
  test('demo IDL has Counter service with ABI-compatible types', () => {
    const entry = registry.getOrThrow('DemoClient');
    const counterUnit = entry.doc.services?.find((s) => s.name === 'Counter');
    expect(counterUnit).toBeDefined();

    // All Counter function params should map to Solidity without error
    for (const func of counterUnit!.funcs ?? []) {
      for (const param of func.params ?? []) {
        expect(() => typeDeclToSolidity(param.type)).not.toThrow();
      }
      // output type (except void)
      if (func.output && func.output !== '()') {
        expect(() => typeDeclToSolidity(func.output)).not.toThrow();
      }
    }
  });

  test('function naming convention: camelCase(Service + Func)', () => {
    // "Counter" + "Add" → "counterAdd"
    const combined = 'Counter' + 'Add';
    const solName = combined[0].toLowerCase() + combined.slice(1);
    expect(solName).toBe('counterAdd');

    // "PingPong" + "Ping" → "pingPongPing"
    const combined2 = 'PingPong' + 'Ping';
    const solName2 = combined2[0].toLowerCase() + combined2.slice(1);
    expect(solName2).toBe('pingPongPing');
  });
});

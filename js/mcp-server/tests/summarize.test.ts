import { describe, test, expect, beforeAll } from 'bun:test';
import { SailsIdlParser } from 'sails-js-parser-idl-v2';
import { registry } from '../src/registry';
import { summarizeProgram, summarizeService, summarizeFunction } from '../src/summarize';
import { readFile } from 'node:fs/promises';
import { resolve, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';

const thisDir = dirname(fileURLToPath(import.meta.url));
const repoRoot = resolve(thisDir, '..', '..', '..');
const demoIdlPath = resolve(repoRoot, 'js/test/demo-v2/demo.idl');

const PROGRAM = 'SummarizeDemo';

beforeAll(async () => {
  const parser = new SailsIdlParser();
  await parser.init();
  const idl = await readFile(demoIdlPath, 'utf-8');
  const doc = parser.parse(idl);
  registry.register(PROGRAM, doc);
});

// ---------------------------------------------------------------------------
// summarizeProgram
// ---------------------------------------------------------------------------

describe('summarizeProgram', () => {
  test('returns program name', () => {
    const summary = summarizeProgram(registry.getOrThrow(PROGRAM));
    expect(summary.program).toBe(PROGRAM);
  });

  test('lists all services', () => {
    const { services } = summarizeProgram(registry.getOrThrow(PROGRAM));
    const names = services.map((s) => s.name);
    expect(names).toContain('Counter');
    expect(names).toContain('Dog');
    expect(names).toContain('ThisThat');
  });

  test('service entry has interface_id, route_idx, function_count, event_count', () => {
    const { services } = summarizeProgram(registry.getOrThrow(PROGRAM));
    const counter = services.find((s) => s.name === 'Counter')!;
    expect(counter.interface_id).toBe('0x579d6daba41b7d82');
    expect(typeof counter.route_idx).toBe('number');
    expect(counter.function_count).toBeGreaterThan(0);
    expect(counter.event_count).toBeGreaterThan(0);
  });

  test('lists constructors', () => {
    const { constructors } = summarizeProgram(registry.getOrThrow(PROGRAM));
    const names = constructors.map((c) => c.name);
    expect(names).toContain('Default');
    expect(names).toContain('New');
  });

  test('constructor params are resolved', () => {
    const { constructors } = summarizeProgram(registry.getOrThrow(PROGRAM));
    const ctor = constructors.find((c) => c.name === 'New')!;
    expect(ctor.params).toHaveLength(2);
    expect(ctor.params[0]).toEqual({ name: 'counter', type: 'Option<u32>' });
    expect(ctor.params[1].name).toBe('dog_position');
  });

  test('type_count sums types across services', () => {
    const { type_count } = summarizeProgram(registry.getOrThrow(PROGRAM));
    // ThisThat has 5 types, References has 1 — at minimum 6
    expect(type_count).toBeGreaterThanOrEqual(6);
  });
});

// ---------------------------------------------------------------------------
// summarizeService
// ---------------------------------------------------------------------------

describe('summarizeService', () => {
  test('Counter: functions and events', () => {
    const detail = summarizeService(registry.getOrThrow(PROGRAM), 'Counter');
    expect(detail.name).toBe('Counter');
    expect(detail.interface_id).toBe('0x579d6daba41b7d82');
    expect(detail.functions.map((f) => f.name)).toEqual(
      expect.arrayContaining(['Add', 'Sub', 'Value']),
    );
    expect(detail.events.map((e) => e.name)).toEqual(
      expect.arrayContaining(['Added', 'Subtracted']),
    );
  });

  test('function params and return_type resolved via TypeResolver', () => {
    const { functions } = summarizeService(registry.getOrThrow(PROGRAM), 'Counter');
    const add = functions.find((f) => f.name === 'Add')!;
    expect(add.params).toEqual([{ name: 'value', type: 'u32' }]);
    expect(add.return_type).toBe('u32');
    expect(add.kind).toBe('command');
  });

  test('void return_type reported as "void"', () => {
    const { functions } = summarizeService(registry.getOrThrow(PROGRAM), 'ThisThat');
    const noop = functions.find((f) => f.name === 'Noop')!;
    expect(noop.return_type).toBe('void');
  });

  test('event fields are resolved', () => {
    const { events } = summarizeService(registry.getOrThrow(PROGRAM), 'Counter');
    const added = events.find((e) => e.name === 'Added')!;
    expect(added.fields.length).toBeGreaterThan(0);
    expect(added.fields[0].type).toBeTruthy();
  });

  test('query kind is reported', () => {
    const { functions } = summarizeService(registry.getOrThrow(PROGRAM), 'Counter');
    const value = functions.find((f) => f.name === 'Value')!;
    expect(value.kind).toBe('query');
  });

  test('ThisThat: struct type is expanded with typed fields', () => {
    const { types } = summarizeService(registry.getOrThrow(PROGRAM), 'ThisThat');
    const doThatParam = types.find((t) => t.name === 'DoThatParam')!;
    expect(doThatParam).toBeDefined();
    expect(doThatParam.kind).toBe('struct');
    // TypeResolver resolves named types to their underlying representation
    const fields = (doThatParam as any).fields as { name: string; type: string }[];
    expect(fields.find((f) => f.name === 'p3')?.type).toBe('ManyVariants');
  });

  test('ThisThat: enum type lists variant names', () => {
    const { types } = summarizeService(registry.getOrThrow(PROGRAM), 'ThisThat');
    const manyVariants = types.find((t) => t.name === 'ManyVariants')!;
    expect(manyVariants.kind).toBe('enum');
    expect((manyVariants as any).variants).toEqual(
      expect.arrayContaining(['One', 'Two', 'Three', 'Four', 'Five', 'Six']),
    );
  });

  test('References: struct alias type', () => {
    const { types } = summarizeService(registry.getOrThrow(PROGRAM), 'References');
    const refCount = types.find((t) => t.name === 'ReferenceCount')!;
    expect(refCount).toBeDefined();
    // ReferenceCount(u32) is a tuple struct, parser may report as struct or alias
    expect(refCount.kind).toMatch(/struct|alias/);
  });

  test('throws on unknown service', () => {
    expect(() =>
      summarizeService(registry.getOrThrow(PROGRAM), 'NoSuchService'),
    ).toThrow(/NoSuchService/);
  });
});

// ---------------------------------------------------------------------------
// summarizeFunction
// ---------------------------------------------------------------------------

describe('summarizeFunction', () => {
  test('returns function detail with service and interface_id', () => {
    const detail = summarizeFunction(registry.getOrThrow(PROGRAM), 'Counter', 'Add');
    expect(detail.name).toBe('Add');
    expect(detail.service).toBe('Counter');
    expect(detail.interface_id).toBe('0x579d6daba41b7d82');
  });

  test('entry_id is sequential index', () => {
    const add = summarizeFunction(registry.getOrThrow(PROGRAM), 'Counter', 'Add');
    const sub = summarizeFunction(registry.getOrThrow(PROGRAM), 'Counter', 'Sub');
    expect(typeof add.entry_id).toBe('number');
    expect(sub.entry_id).toBe(add.entry_id + 1);
  });

  test('params and return_type resolved by TypeResolver', () => {
    const detail = summarizeFunction(registry.getOrThrow(PROGRAM), 'Counter', 'Add');
    expect(detail.params).toEqual([{ name: 'value', type: 'u32' }]);
    expect(detail.return_type).toBe('u32');
  });

  test('multi-param function', () => {
    const detail = summarizeFunction(registry.getOrThrow(PROGRAM), 'ThisThat', 'DoThis');
    expect(detail.params[0]).toEqual({ name: 'p1', type: 'u32' });
    expect(detail.params[1]).toEqual({ name: 'p2', type: 'String' });
  });

  test('throws on unknown function', () => {
    expect(() =>
      summarizeFunction(registry.getOrThrow(PROGRAM), 'Counter', 'NoSuchFunc'),
    ).toThrow(/NoSuchFunc/);
  });

  test('throws on unknown service', () => {
    expect(() =>
      summarizeFunction(registry.getOrThrow(PROGRAM), 'NoSuchService', 'Add'),
    ).toThrow(/NoSuchService/);
  });
});

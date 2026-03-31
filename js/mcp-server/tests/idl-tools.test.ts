import { describe, test, expect, beforeAll } from 'bun:test';
import { SailsIdlParser } from 'sails-js-parser-idl-v2';
import { registry } from '../src/registry';
import { summarizeProgram, summarizeService, summarizeFunction } from '../src/summarize';
import { readFile } from 'node:fs/promises';
import { resolve, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';

const thisDir = dirname(fileURLToPath(import.meta.url));
const repoRoot = resolve(thisDir, '..', '..', '..');
// Use demo-v2 IDL which is compatible with the current parser WASM
const demoIdlPath = resolve(repoRoot, 'js/test/demo-v2/demo.idl');

let parser: SailsIdlParser;

beforeAll(async () => {
  parser = new SailsIdlParser();
  await parser.init();
});

describe('IDL parsing', () => {
  test('parse demo IDL', async () => {
    const idl = await readFile(demoIdlPath, 'utf-8');
    const doc = parser.parse(idl);

    expect(doc).toBeDefined();
    expect(doc.program).toBeDefined();
    expect(doc.program!.name).toBe('DemoClient');
    expect(doc.services!.length).toBeGreaterThan(0);
  });

  test('register and summarize program', async () => {
    const idl = await readFile(demoIdlPath, 'utf-8');
    const doc = parser.parse(idl);

    const entry = registry.register('DemoClient', doc);
    const summary = summarizeProgram(entry);

    expect(summary.program).toBe('DemoClient');
    expect(summary.services.length).toBeGreaterThan(0);
    expect(summary.constructors.length).toBe(2); // Default, New

    // Check Counter service exists
    const counter = summary.services.find((s) => s.name === 'Counter');
    expect(counter).toBeDefined();
    expect(counter!.interface_id).toBe('0x579d6daba41b7d82');
  });

  test('summarize Counter service', async () => {
    const entry = registry.getOrThrow('DemoClient');
    const detail = summarizeService(entry, 'Counter');

    expect(detail.name).toBe('Counter');
    expect(detail.interface_id).toBe('0x579d6daba41b7d82');
    expect(detail.functions.length).toBe(3); // Add, Sub, Value
    expect(detail.events.length).toBe(2); // Added, Subtracted

    const addFn = detail.functions.find((f) => f.name === 'Add');
    expect(addFn).toBeDefined();
    expect(addFn!.params).toEqual([{ name: 'value', type: 'u32' }]);
    expect(addFn!.return_type).toBe('u32');
  });

  test('summarize function detail', async () => {
    const entry = registry.getOrThrow('DemoClient');
    const detail = summarizeFunction(entry, 'Counter', 'Add');

    expect(detail.name).toBe('Add');
    expect(detail.service).toBe('Counter');
    expect(detail.params).toEqual([{ name: 'value', type: 'u32' }]);
    expect(detail.return_type).toBe('u32');
    expect(detail.entry_id).toBe(0);
  });

  test('validate valid IDL', () => {
    const idl = `
      service Foo {
        functions {
          Bar() -> u32;
        }
      }
    `;
    expect(() => parser.parse(idl)).not.toThrow();
  });

  test('validate invalid IDL', () => {
    const idl = 'this is not valid IDL at all!!!';
    expect(() => parser.parse(idl)).toThrow();
  });
});

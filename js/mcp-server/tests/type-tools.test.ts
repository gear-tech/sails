import { describe, test, expect, beforeAll } from 'bun:test';
import { SailsIdlParser } from 'sails-js-parser-idl-v2';
import { registry } from '../src/registry';
import { readFile } from 'node:fs/promises';
import { resolve, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';
import { registerTypeTools } from '../src/tools/type-tools';
import { McpServer } from '@modelcontextprotocol/sdk/server/mcp.js';

const thisDir = dirname(fileURLToPath(import.meta.url));
const repoRoot = resolve(thisDir, '..', '..', '..');
const demoIdlPath = resolve(repoRoot, 'js/test/demo-v2/demo.idl');

const PROGRAM = 'TypeToolsDemo';

// Call a registered tool directly by invoking its handler via the server
type ToolResult = { isError?: boolean; content: { type: string; text: string }[] };

function makeServer() {
  const server = new McpServer({ name: 'test', version: '0.0.0' });
  registerTypeTools(server);
  return server;
}

async function callTool(server: McpServer, name: string, args: Record<string, unknown>): Promise<ToolResult> {
  const result = await (server as any)._registeredTools[name].handler(args);
  return result as ToolResult;
}

function json(result: ToolResult) {
  return JSON.parse(result.content[0].text);
}

let server: McpServer;

beforeAll(async () => {
  const parser = new SailsIdlParser();
  await parser.init();
  const idl = await readFile(demoIdlPath, 'utf-8');
  const doc = parser.parse(idl);
  registry.register(PROGRAM, doc);
  server = makeServer();
});

// ---------------------------------------------------------------------------
// sails_resolve_type
// ---------------------------------------------------------------------------

describe('sails_resolve_type', () => {
  test('resolves a struct: name, scope, definition.kind, definition.fields', async () => {
    const result = await callTool(server, 'sails_resolve_type', {
      program: PROGRAM,
      type_name: 'DoThatParam',
      service: 'ThisThat',
    });
    expect(result.isError).toBeFalsy();
    const { name, scope, definition } = json(result);
    expect(name).toBe('DoThatParam');
    expect(scope).toBe('ThisThat');
    expect(definition.kind).toBe('struct');
    expect(Array.isArray(definition.fields)).toBe(true);
    const fieldNames = definition.fields.map((f: any) => f.name);
    expect(fieldNames).toEqual(expect.arrayContaining(['p1', 'p2', 'p3']));
  });

  test('resolves an enum: definition.kind and definition.variants', async () => {
    const result = await callTool(server, 'sails_resolve_type', {
      program: PROGRAM,
      type_name: 'ManyVariants',
      service: 'ThisThat',
    });
    expect(result.isError).toBeFalsy();
    const { name, scope, definition } = json(result);
    expect(name).toBe('ManyVariants');
    expect(scope).toBe('ThisThat');
    expect(definition.kind).toBe('enum');
    expect(Array.isArray(definition.variants)).toBe(true);
    const variantNames = definition.variants.map((v: any) => v.name);
    expect(variantNames).toEqual(expect.arrayContaining(['One', 'Two', 'Three', 'Four', 'Five', 'Six']));
  });

  test('auto-discovers type across all services; struct definition present', async () => {
    const result = await callTool(server, 'sails_resolve_type', {
      program: PROGRAM,
      type_name: 'ReferenceCount',
    });
    expect(result.isError).toBeFalsy();
    const { name, scope, definition } = json(result);
    expect(name).toBe('ReferenceCount');
    expect(scope).toBe('References');
    // ReferenceCount(u32) is a tuple struct
    expect(definition.kind).toBe('struct');
    expect(Array.isArray(definition.fields)).toBe(true);
  });

  test('returns error for unknown type', async () => {
    const result = await callTool(server, 'sails_resolve_type', {
      program: PROGRAM,
      type_name: 'NoSuchType',
    });
    expect(result.isError).toBe(true);
    expect(result.content[0].text).toMatch(/NoSuchType/);
  });

  test('returns error for unknown program', async () => {
    const result = await callTool(server, 'sails_resolve_type', {
      program: 'NoSuchProgram',
      type_name: 'DoThatParam',
    });
    expect(result.isError).toBe(true);
    expect(result.content[0].text).toMatch(/NoSuchProgram/);
  });
});

// ---------------------------------------------------------------------------
// sails_encode_type
// ---------------------------------------------------------------------------

describe('sails_encode_type', () => {
  test('encodes a u32 value', async () => {
    const result = await callTool(server, 'sails_encode_type', {
      program: PROGRAM,
      service: 'Counter',
      type_name: 'u32',
      value: 42,
    });
    expect(result.isError).toBeFalsy();
    const data = json(result);
    expect(data.type).toBe('u32');
    expect(data.hex).toMatch(/^0x/);
  });

  test('encodes a bool value', async () => {
    const result = await callTool(server, 'sails_encode_type', {
      program: PROGRAM,
      service: 'Counter',
      type_name: 'bool',
      value: true,
    });
    expect(result.isError).toBeFalsy();
    expect(json(result).hex).toBe('0x01');
  });

  test('returns error for unknown service', async () => {
    const result = await callTool(server, 'sails_encode_type', {
      program: PROGRAM,
      service: 'NoSuchService',
      type_name: 'u32',
      value: 1,
    });
    expect(result.isError).toBe(true);
    expect(result.content[0].text).toMatch(/NoSuchService/);
  });

  test('returns error for unknown program', async () => {
    const result = await callTool(server, 'sails_encode_type', {
      program: 'NoSuchProgram',
      service: 'Counter',
      type_name: 'u32',
      value: 1,
    });
    expect(result.isError).toBe(true);
  });
});

// ---------------------------------------------------------------------------
// sails_decode_type
// ---------------------------------------------------------------------------

describe('sails_decode_type', () => {
  test('decodes a u32 value', async () => {
    // First encode to get the canonical hex, then decode it back
    const encoded = await callTool(server, 'sails_encode_type', {
      program: PROGRAM,
      service: 'Counter',
      type_name: 'u32',
      value: 42,
    });
    const hex = json(encoded).hex;
    const result = await callTool(server, 'sails_decode_type', {
      program: PROGRAM,
      service: 'Counter',
      type_name: 'u32',
      hex,
    });
    expect(result.isError).toBeFalsy();
    const data = json(result);
    expect(data.type).toBe('u32');
    expect(data.value).toBe(42);
  });

  test('decodes a bool value', async () => {
    const result = await callTool(server, 'sails_decode_type', {
      program: PROGRAM,
      service: 'Counter',
      type_name: 'bool',
      hex: '0x01',
    });
    expect(result.isError).toBeFalsy();
    expect(json(result).value).toBe(true);
  });

  test('encode/decode round-trip for u32', async () => {
    const encoded = await callTool(server, 'sails_encode_type', {
      program: PROGRAM,
      service: 'Counter',
      type_name: 'u32',
      value: 12345,
    });
    const { hex } = json(encoded);

    const decoded = await callTool(server, 'sails_decode_type', {
      program: PROGRAM,
      service: 'Counter',
      type_name: 'u32',
      hex,
    });
    expect(json(decoded).value).toBe(12345);
  });

  test('returns error for unknown service', async () => {
    const result = await callTool(server, 'sails_decode_type', {
      program: PROGRAM,
      service: 'NoSuchService',
      type_name: 'u32',
      hex: '0x00000000',
    });
    expect(result.isError).toBe(true);
    expect(result.content[0].text).toMatch(/NoSuchService/);
  });
});

import { describe, test, expect, beforeAll } from 'bun:test';
import { McpServer } from '@modelcontextprotocol/sdk/server/mcp.js';
import { SailsIdlParser, SailsMessageHeader, InterfaceId } from 'sails-js-parser-idl-v2';
import { registry } from '../src/registry';
import { registerCodecTools } from '../src/tools/codec-tools';
import { registerUtilTools } from '../src/tools/util-tools';
import { readFile } from 'node:fs/promises';
import { resolve, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';

const thisDir = dirname(fileURLToPath(import.meta.url));
const repoRoot = resolve(thisDir, '..', '..', '..');
const demoIdlPath = resolve(repoRoot, 'js/test/demo-v2/demo.idl');

const PROGRAM = 'ToolRoutingDemo';

type ToolResult = { isError?: boolean; content: { type: string; text: string }[] };

function makeServer() {
  const server = new McpServer({ name: 'test', version: '0.0.0' });
  registerCodecTools(server);
  registerUtilTools(server);
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

describe('tool routing', () => {
  test('sails_explain_payload identifies the matching function header', async () => {
    const entry = registry.getOrThrow(PROGRAM);
    const payload = entry.program.services['Counter'].functions['Sub'].encodePayload(42);

    const result = await callTool(server, 'sails_explain_payload', {
      program: PROGRAM,
      hex: payload,
    });

    expect(result.isError).toBeFalsy();
    expect(json(result)).toMatchObject({
      identified: true,
      service: 'Counter',
      function: 'Sub',
      kind: 'command',
      args: { value: 42 },
    });
  });

  test('sails_decode_event auto-detects the matching event header', async () => {
    const entry = registry.getOrThrow(PROGRAM);
    const counter = entry.program.services['Counter'];
    const header = SailsMessageHeader.v1(
      InterfaceId.fromString('0x579d6daba41b7d82'),
      1,
      counter.routeIdx,
    );
    const payload = counter.registry
      .createType('([u8; 16], u32)', [
        header.toBytes(),
        7,
      ])
      .toHex();

    const result = await callTool(server, 'sails_decode_event', {
      program: PROGRAM,
      service: 'Counter',
      hex: payload,
    });

    expect(result.isError).toBeFalsy();
    expect(json(result)).toMatchObject({
      event: 'Subtracted',
      data: 7,
      auto_detected: true,
    });
  });
});

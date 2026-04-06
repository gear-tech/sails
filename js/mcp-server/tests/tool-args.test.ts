import { describe, test, expect } from 'bun:test';
import { z } from 'zod';
import { McpServer } from '@modelcontextprotocol/sdk/server/mcp.js';
import { registerAbiTools } from '../src/tools/abi-tools';
import { registerCodecTools } from '../src/tools/codec-tools';
import { registerHeaderTools } from '../src/tools/header-tools';
import { registerIdlTools } from '../src/tools/idl-tools';
import { registerSourceTools } from '../src/tools/source-tools';
import { registerTypeTools } from '../src/tools/type-tools';
import { registerUtilTools } from '../src/tools/util-tools';

type ToolListResult = {
  tools: Array<{
    name: string;
    inputSchema?: {
      properties?: Record<string, unknown>;
    };
  }>;
};

function makeServer() {
  const server = new McpServer({ name: 'test', version: '0.0.0' });
  registerAbiTools(server);
  registerCodecTools(server);
  registerHeaderTools(server);
  registerIdlTools(server);
  registerSourceTools(server);
  registerTypeTools(server);
  registerUtilTools(server);
  return server;
}

async function listTools(server: McpServer): Promise<ToolListResult> {
  const result = await (server.server as any)._requestHandlers.get('tools/list')(
    {
      jsonrpc: '2.0',
      id: 1,
      method: 'tools/list',
      params: {},
    },
    {},
  );

  return result as ToolListResult;
}

describe('tool arg names', () => {
  test('all tool argument names survive MCP record parsing', async () => {
    const server = makeServer();
    const listed = await listTools(server);
    const recordSchema = z.record(z.string(), z.unknown());

    const dropped: Array<{ tool: string; expected: string[]; actual: string[] }> = [];

    for (const tool of listed.tools) {
      const argNames = Object.keys(tool.inputSchema?.properties ?? {});
      if (argNames.length === 0) {
        continue;
      }

      const sampleArgs = Object.fromEntries(argNames.map((name) => [name, true]));
      const parsed = recordSchema.safeParse(sampleArgs);
      expect(parsed.success).toBeTrue();

      if (!parsed.success) {
        continue;
      }

      const actualNames = Object.keys(parsed.data);
      if (actualNames.length !== argNames.length || actualNames.some((name, i) => name !== argNames[i])) {
        dropped.push({
          tool: tool.name,
          expected: argNames,
          actual: actualNames,
        });
      }
    }

    expect(dropped).toEqual([]);
  });
});

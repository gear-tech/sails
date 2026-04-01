import type { McpServer } from '@modelcontextprotocol/sdk/server/mcp.js';
import { z } from 'zod';
import { SailsIdlParser } from 'sails-js-parser-idl-v2';
import { registry } from '../registry.js';
import { summarizeProgram, summarizeService, summarizeFunction } from '../summarize.js';

let parser: SailsIdlParser | null = null;

export async function getParser(): Promise<SailsIdlParser> {
  if (!parser) {
    parser = new SailsIdlParser();
    await parser.init();
  }
  return parser;
}

export function registerIdlTools(server: McpServer) {
  server.registerTool(
    'sails_parse_idl',
    {
      description:
        'Parse Sails IDL v2 text and register the program for subsequent encoding/decoding operations. ' +
        'Returns a structured summary of services, constructors, types, events, and interface IDs. ' +
        'Does NOT resolve !@include directives — use sails_load_idl for IDL files with includes.',
      inputSchema: {
        idl: z.string().describe('Raw IDL v2 text'),
        program: z
          .string()
          .optional()
          .describe('Name to register the program under. Defaults to the program name from IDL.'),
      },
      annotations: { readOnlyHint: true },
    },
    async ({ idl, program }) => {
      try {
        const p = await getParser();
        const doc = p.parse(idl);

        const name = program ?? doc.program?.name ?? 'unnamed';
        const entry = registry.register(name, doc);
        const summary = summarizeProgram(entry);

        return {
          content: [{ type: 'text', text: JSON.stringify(summary, null, 2) }],
        };
      } catch (error: any) {
        return {
          isError: true,
          content: [{ type: 'text', text: `IDL parse error: ${error.message}` }],
        };
      }
    },
  );

  server.registerTool(
    'sails_validate_idl',
    {
      description:
        'Validate Sails IDL v2 text without registering it. Returns parse errors and warnings. ' +
        'Use this to check IDL syntax before committing or deploying.',
      inputSchema: {
        idl: z.string().describe('Raw IDL v2 text to validate'),
      },
      annotations: { readOnlyHint: true },
    },
    async ({ idl }) => {
      try {
        const p = await getParser();
        p.parse(idl);
        return {
          content: [{ type: 'text', text: JSON.stringify({ valid: true, errors: null }, null, 2) }],
        };
      } catch (error: any) {
        return {
          content: [
            {
              type: 'text',
              text: JSON.stringify(
                { valid: false, errors: [error.message] },
                null,
                2,
              ),
            },
          ],
        };
      }
    },
  );

  server.registerTool(
    'sails_inspect_service',
    {
      description:
        'Get detailed information about a specific service including all functions, queries, events, types, ' +
        'extended services, and its interface ID. Requires the program to be parsed first via sails_parse_idl.',
      inputSchema: {
        program: z.string().describe('Registered program name'),
        service: z.string().describe('Service name (e.g. "Counter")'),
      },
      annotations: { readOnlyHint: true },
    },
    async ({ program: programName, service: serviceName }) => {
      try {
        const entry = registry.getOrThrow(programName);
        const detail = summarizeService(entry, serviceName);
        return {
          content: [{ type: 'text', text: JSON.stringify(detail, null, 2) }],
        };
      } catch (error: any) {
        return {
          isError: true,
          content: [{ type: 'text', text: error.message }],
        };
      }
    },
  );

  server.registerTool(
    'sails_inspect_function',
    {
      description:
        'Get detailed information about a specific function or query, including parameter types, ' +
        'return type, SCALE codec type strings, entry ID, and documentation. ' +
        'Requires the program to be parsed first via sails_parse_idl.',
      inputSchema: {
        program: z.string().describe('Registered program name'),
        service: z.string().describe('Service name'),
        function: z.string().describe('Function or query name'),
      },
      annotations: { readOnlyHint: true },
    },
    async ({ program: programName, service: serviceName, function: funcName }) => {
      try {
        const entry = registry.getOrThrow(programName);
        const detail = summarizeFunction(entry, serviceName, funcName);
        return {
          content: [{ type: 'text', text: JSON.stringify(detail, null, 2) }],
        };
      } catch (error: any) {
        return {
          isError: true,
          content: [{ type: 'text', text: error.message }],
        };
      }
    },
  );
}

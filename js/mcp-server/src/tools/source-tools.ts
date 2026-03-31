import type { McpServer } from '@modelcontextprotocol/sdk/server/mcp.js';
import { z } from 'zod';
import { registry } from '../registry.js';
import { preprocessIdl } from '../idl-loader.js';
import { extractIdlFromWasm } from '../wasm-extractor.js';
import { getParser } from './idl-tools.js';
import { summarizeProgram } from '../summarize.js';

export function registerSourceTools(server: McpServer) {
  server.registerTool(
    'sails_load_idl',
    {
      description:
        'Load an IDL file from a file path, resolving !@include directives recursively ' +
        '(supports local files and git:// URLs). Preprocesses into a single IDL text, ' +
        'then parses and registers the program for subsequent operations. ' +
        'Use this instead of sails_parse_idl when your IDL has !@include statements.',
      inputSchema: {
        path: z.string().describe('Path to the .idl file (absolute or relative)'),
        program: z
          .string()
          .optional()
          .describe('Name to register the program under. Defaults to the program name from IDL.'),
      },
      annotations: { readOnlyHint: true },
    },
    async ({ path, program }) => {
      try {
        const preprocessed = await preprocessIdl(path);
        const p = await getParser();
        const doc = p.parse(preprocessed);

        const name = program ?? doc.program?.name ?? 'unnamed';
        const entry = registry.register(name, doc);
        const summary = summarizeProgram(entry);

        return {
          content: [{ type: 'text', text: JSON.stringify(summary, null, 2) }],
        };
      } catch (err: any) {
        return {
          isError: true,
          content: [{ type: 'text', text: `Load IDL error: ${err.message}` }],
        };
      }
    },
  );

  server.registerTool(
    'sails_extract_idl_from_wasm',
    {
      description:
        'Extract embedded IDL from a .wasm binary\'s "sails:idl" custom section. ' +
        'Returns the raw IDL text which can then be passed to sails_parse_idl. ' +
        'The IDL may be deflate-compressed inside the WASM. ' +
        'Useful for debugging deployed programs when you have the .wasm but not the .idl file.',
      inputSchema: {
        wasm_path: z.string().describe('Path to the .wasm binary file'),
      },
      annotations: { readOnlyHint: true },
    },
    async ({ wasm_path }) => {
      try {
        const idl = await extractIdlFromWasm(wasm_path);
        if (idl === null) {
          return {
            isError: true,
            content: [
              {
                type: 'text',
                text: 'No "sails:idl" custom section found in the WASM binary. ' +
                  'The program may not have been built with IDL embedding (cargo sails idl-embed).',
              },
            ],
          };
        }
        return {
          content: [
            {
              type: 'text',
              text: JSON.stringify({ idl, length: idl.length }, null, 2),
            },
          ],
        };
      } catch (err: any) {
        return {
          isError: true,
          content: [{ type: 'text', text: `WASM extract error: ${err.message}` }],
        };
      }
    },
  );
}

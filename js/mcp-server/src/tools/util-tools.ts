import type { McpServer } from '@modelcontextprotocol/sdk/server/mcp.js';
import { z } from 'zod';
import { MAGIC_BYTES, SailsMessageHeader } from 'sails-js-parser-idl-v2';
import { registry } from '../registry.js';

function hexToBytes(hex: string): Uint8Array {
  let h = hex.startsWith('0x') ? hex.slice(2) : hex;
  if (h.length % 2 !== 0) h = '0' + h;
  const bytes = new Uint8Array(h.length / 2);
  for (let i = 0; i < bytes.length; i++) {
    bytes[i] = Number.parseInt(h.slice(i * 2, i * 2 + 2), 16);
  }
  return bytes;
}

export function registerUtilTools(server: McpServer) {
  server.registerTool(
    'sails_list_programs',
    {
      description:
        'List all programs currently registered in the MCP session. ' +
        'Shows program names, service counts, and constructor counts.',
      inputSchema: {},
      annotations: { readOnlyHint: true },
    },
    async () => {
      const programs = registry.list().map((entry) => ({
        program: entry.name,
        services: Object.keys(entry.program.services),
        constructors: entry.program.ctors ? Object.keys(entry.program.ctors) : [],
      }));

      if (programs.length === 0) {
        return {
          content: [
            {
              type: 'text',
              text: 'No programs registered. Use sails_parse_idl or sails_load_idl to register a program first.',
            },
          ],
        };
      }

      return {
        content: [{ type: 'text', text: JSON.stringify(programs, null, 2) }],
      };
    },
  );

  server.registerTool(
    'sails_detect_encoding',
    {
      description:
        'Detect whether a hex payload uses Sails Message Header (SCALE encoding) or ' +
        'ABI encoding (4-byte selector). Checks for the "GM" (0x474D) magic bytes. ' +
        'No program registration required.',
      inputSchema: {
        hex: z.string().describe('Hex-encoded payload to analyze'),
      },
      annotations: { readOnlyHint: true },
    },
    async ({ hex }) => {
      try {
        const bytes = hexToBytes(hex);

        if (bytes.length < 4) {
          return {
            content: [
              {
                type: 'text',
                text: JSON.stringify(
                  {
                    encoding: 'unknown',
                    reason: 'Payload too short (< 4 bytes)',
                  },
                  null,
                  2,
                ),
              },
            ],
          };
        }

        // Check for Sails magic bytes "GM" (0x47 0x4D)
        if (bytes[0] === MAGIC_BYTES[0] && bytes[1] === MAGIC_BYTES[1]) {
          const details: any = { encoding: 'scale', magic: 'GM (0x474D)' };

          if (bytes.length >= 16) {
            try {
              const { header } = SailsMessageHeader.tryReadBytes(bytes, 0);
              details.version = header.version;
              details.interface_id = header.interfaceId.toString();
              details.entry_id = header.entryId;
              details.route_idx = header.routeIdx;
            } catch {
              details.note = 'Magic bytes present but header parse failed';
            }
          }

          return {
            content: [{ type: 'text', text: JSON.stringify(details, null, 2) }],
          };
        }

        // No Sails magic - likely ABI encoding (4-byte selector)
        const selector = `0x${Array.from(bytes.subarray(0, 4), (byte) => byte.toString(16).padStart(2, '0')).join('')}`;
        return {
          content: [
            {
              type: 'text',
              text: JSON.stringify(
                {
                  encoding: 'abi',
                  selector,
                  note: 'No Sails "GM" magic bytes. Likely ABI-encoded (ethexe) with 4-byte function selector.',
                  payload_length: bytes.length,
                },
                null,
                2,
              ),
            },
          ],
        };
      } catch (error: any) {
        return {
          isError: true,
          content: [{ type: 'text', text: `Detection error: ${error.message}` }],
        };
      }
    },
  );

  server.registerTool(
    'sails_explain_payload',
    {
      description:
        'Given a hex payload and a registered program, try to identify which service and function ' +
        "it targets by matching the header's interface_id and entry_id. " +
        'Decodes the arguments if a match is found. Works for SCALE-encoded payloads with Sails headers.',
      inputSchema: {
        program: z.string().describe('Registered program name'),
        hex: z.string().describe('Hex-encoded payload to identify'),
      },
      annotations: { readOnlyHint: true },
    },
    async ({ program: programName, hex }) => {
      try {
        const bytes = hexToBytes(hex);
        const entry = registry.getOrThrow(programName);

        // Try to parse the Sails header
        const { ok, header } = SailsMessageHeader.tryFromBytes(bytes);
        if (!ok || !header) {
          return {
            content: [
              {
                type: 'text',
                text: JSON.stringify(
                  {
                    identified: false,
                    reason: 'Not a valid Sails message (no "GM" magic bytes or invalid header)',
                  },
                  null,
                  2,
                ),
              },
            ],
          };
        }

        const targetEntryId = header.entryId;

        // Search all services for a match
        for (const [serviceName, service] of Object.entries(entry.program.services)) {
          // Check functions
          for (const [funcName, func] of Object.entries(service.functions)) {
            try {
              const decoded = func.decodePayload(hex as `0x${string}`);
              // If decodePayload succeeds without error, it's a match
              return {
                content: [
                  {
                    type: 'text',
                    text: JSON.stringify(
                      {
                        identified: true,
                        service: serviceName,
                        function: funcName,
                        kind: 'command',
                        interface_id: header.interfaceId.toString(),
                        entry_id: targetEntryId,
                        args: decoded,
                      },
                      null,
                      2,
                    ),
                  },
                ],
              };
            } catch {
              continue;
            }
          }

          // Check queries
          for (const [queryName, query] of Object.entries(service.queries)) {
            try {
              const decoded = query.decodePayload(hex as `0x${string}`);
              return {
                content: [
                  {
                    type: 'text',
                    text: JSON.stringify(
                      {
                        identified: true,
                        service: serviceName,
                        function: queryName,
                        kind: 'query',
                        interface_id: header.interfaceId.toString(),
                        entry_id: targetEntryId,
                        args: decoded,
                      },
                      null,
                      2,
                    ),
                  },
                ],
              };
            } catch {
              continue;
            }
          }
        }

        return {
          content: [
            {
              type: 'text',
              text: JSON.stringify(
                {
                  identified: false,
                  interface_id: header.interfaceId.toString(),
                  entry_id: targetEntryId,
                  route_idx: header.routeIdx,
                  reason: `No matching service/function found for interface_id=${header.interfaceId.toString()} entry_id=${targetEntryId}`,
                },
                null,
                2,
              ),
            },
          ],
        };
      } catch (error: any) {
        return {
          isError: true,
          content: [{ type: 'text', text: `Explain error: ${error.message}` }],
        };
      }
    },
  );
}

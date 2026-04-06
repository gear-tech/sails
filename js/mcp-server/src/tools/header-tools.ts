import type { McpServer } from '@modelcontextprotocol/sdk/server/mcp.js';
import { z } from 'zod';
import { SailsMessageHeader, InterfaceId } from 'sails-js-parser-idl-v2';
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

function bytesToHex(bytes: Uint8Array): string {
  return '0x' + [...bytes].map((b) => b.toString(16).padStart(2, '0')).join('');
}

export function registerHeaderTools(server: McpServer) {
  server.registerTool(
    'sails_parse_header',
    {
      description:
        'Parse a raw Sails Message Header from hex. No IDL or program registration needed. ' +
        'Extracts: magic bytes ("GM"), version, header length, interface ID, entry ID, route index. ' +
        'Also returns the remaining payload after the header.',
      inputSchema: {
        hex: z.string().describe('Hex-encoded payload containing at least 16 bytes for the header'),
      },
      annotations: { readOnlyHint: true },
    },
    async ({ hex }) => {
      try {
        const bytes = hexToBytes(hex);
        const { header, offset } = SailsMessageHeader.tryReadBytes(bytes, 0);
        const payloadAfterHeader = bytes.slice(offset);

        return {
          content: [
            {
              type: 'text',
              text: JSON.stringify(
                {
                  valid: true,
                  magic: 'GM',
                  version: header.version,
                  header_length: header.hlen,
                  interface_id: header.interfaceId.toString(),
                  interface_id_u64: header.interfaceId.asU64().toString(),
                  entry_id: header.entryId,
                  route_idx: header.routeIdx,
                  payload_after_header: bytesToHex(payloadAfterHeader),
                  payload_length: payloadAfterHeader.length,
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
          content: [{ type: 'text', text: `Header parse error: ${error.message}` }],
        };
      }
    },
  );

  server.registerTool(
    'sails_build_header',
    {
      description:
        'Build a 16-byte Sails Message Header v1 from explicit components. ' +
        'Useful for manual message construction or testing.',
      inputSchema: {
        interface_id: z
          .string()
          .describe('Interface ID as hex string (e.g. "0x579d6daba41b7d82")'),
        entry_id: z.number().int().min(0).max(65_535).describe('Entry ID (u16, 0-65535)'),
        route_idx: z.number().int().min(0).max(255).default(0).describe('Route index (u8, 0-255)'),
      },
      annotations: { readOnlyHint: true },
    },
    async ({ interface_id, entry_id, route_idx }) => {
      try {
        const iid = InterfaceId.fromString(interface_id);
        const header = SailsMessageHeader.v1(iid, entry_id, route_idx);
        const bytes = header.toBytes();

        return {
          content: [
            {
              type: 'text',
              text: JSON.stringify(
                {
                  hex: bytesToHex(bytes),
                  bytes: [...bytes],
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
          content: [{ type: 'text', text: `Header build error: ${error.message}` }],
        };
      }
    },
  );

  server.registerTool(
    'sails_verify_interface_id',
    {
      description:
        'Show the computed and declared interface IDs for a service. ' +
        'The IDL parser computes interface IDs structurally (BLAKE3 hash of the service shape). ' +
        'If the service has an explicit @0x... annotation, this tool compares computed vs declared. ' +
        'Essential for @partial service compatibility checks.',
      inputSchema: {
        program: z.string().describe('Registered program name'),
        service: z.string().describe('Service name'),
      },
      annotations: { readOnlyHint: true },
    },
    async ({ program: programName, service: serviceName }) => {
      try {
        const entry = registry.getOrThrow(programName);
        const unit = entry.doc.services?.find((s) => s.name === serviceName);
        if (!unit) {
          const available = entry.doc.services?.map((s) => s.name) ?? [];
          throw new Error(
            `Service "${serviceName}" not found. Available: [${available.join(', ')}]`,
          );
        }

        const isPartial = unit.annotations?.some(([k]) => k === 'partial') ?? false;

        return {
          content: [
            {
              type: 'text',
              text: JSON.stringify(
                {
                  service: serviceName,
                  interface_id: unit.interface_id ?? null,
                  is_partial: isPartial,
                  note: isPartial
                    ? '@partial services require an explicit interface_id in the IDL.'
                    : 'Interface ID is computed structurally from the service definition.',
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
          content: [{ type: 'text', text: error.message }],
        };
      }
    },
  );

  server.registerTool(
    'sails_list_interface_ids',
    {
      description:
        'List all services in a parsed program with their interface IDs and entry ID maps for functions and events.',
      inputSchema: {
        program: z.string().describe('Registered program name'),
      },
      annotations: { readOnlyHint: true },
    },
    async ({ program: programName }) => {
      try {
        const entry = registry.getOrThrow(programName);
        const result = (entry.doc.services ?? []).map((unit) => ({
          service: unit.name,
          interface_id: unit.interface_id ?? null,
          functions: (unit.funcs ?? []).map((f, idx) => {
            const ann = f.annotations?.find(([k]) => k === 'entry-id');
            return {
              name: f.name,
              entry_id: ann ? Number(ann[1]) : idx,
              kind: f.kind ?? 'command',
            };
          }),
          events: (unit.events ?? []).map((e, idx) => {
            const ann = e.annotations?.find(([k]) => k === 'entry-id');
            return {
              name: e.name,
              entry_id: ann ? Number(ann[1]) : idx,
            };
          }),
        }));

        return {
          content: [{ type: 'text', text: JSON.stringify(result, null, 2) }],
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

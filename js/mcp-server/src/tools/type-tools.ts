import type { McpServer } from '@modelcontextprotocol/sdk/server/mcp.js';
import { z } from 'zod';
import { registry } from '../registry.js';

export function registerTypeTools(server: McpServer) {
  server.registerTool(
    'sails_resolve_type',
    {
      description:
        'Resolve a user-defined type from the IDL into its full definition. ' +
        'Shows struct fields, enum variants, and how the type maps to SCALE codec. ' +
        'If the type is service-scoped, specify the service name.',
      inputSchema: {
        program: z.string().describe('Registered program name'),
        type_name: z.string().describe('Type name (e.g. "DoThatParam", "ManyVariants")'),
        service: z.string().optional().describe('Service name (for service-scoped types)'),
      },
      annotations: { readOnlyHint: true },
    },
    async ({ program: programName, type_name, service: serviceName }) => {
      try {
        const entry = registry.getOrThrow(programName);
        const doc = entry.doc;

        // Search in service-scoped types first, then program-level types
        const searchScopes: Array<{ scope: string; types: any[] }> = [];

        if (serviceName) {
          const unit = doc.services?.find((s) => s.name === serviceName);
          if (unit?.types) {
            searchScopes.push({ scope: serviceName, types: unit.types });
          }
        } else {
          // Search all scopes
          if (doc.program?.types) {
            searchScopes.push({ scope: 'program', types: doc.program.types });
          }
          for (const svc of doc.services ?? []) {
            if (svc.types) {
              searchScopes.push({ scope: svc.name, types: svc.types });
            }
          }
        }

        for (const { scope, types } of searchScopes) {
          const found = types.find((t: any) => t.name === type_name);
          if (found) {
            return {
              content: [
                {
                  type: 'text',
                  text: JSON.stringify(
                    {
                      name: found.name,
                      scope,
                      definition: found.def,
                    },
                    null,
                    2,
                  ),
                },
              ],
            };
          }
        }

        // Collect all available type names for the error message
        const allTypes: string[] = [];
        for (const { types } of searchScopes) {
          for (const t of types) {
            allTypes.push(t.name);
          }
        }

        throw new Error(
          `Type "${type_name}" not found. Available types: [${allTypes.join(', ')}]`,
        );
      } catch (err: any) {
        return {
          isError: true,
          content: [{ type: 'text', text: err.message }],
        };
      }
    },
  );

  server.registerTool(
    'sails_encode_type',
    {
      description:
        'SCALE-encode a JSON value for a specific IDL type. Useful for constructing payload fragments ' +
        'or testing type encoding independently. Requires specifying the service context for type resolution.',
      inputSchema: {
        program: z.string().describe('Registered program name'),
        service: z.string().describe('Service name (for type resolution context)'),
        type_name: z.string().describe('Type name or SCALE type expression (e.g. "u32", "DoThatParam")'),
        value: z.any().describe('JSON value to encode'),
      },
      annotations: { readOnlyHint: true },
    },
    async ({ program: programName, service: serviceName, type_name, value }) => {
      try {
        const entry = registry.getOrThrow(programName);
        const services = entry.program.services;
        const service = services[serviceName];
        if (!service) {
          throw new Error(`Service "${serviceName}" not found`);
        }

        const encoded = service.registry.createType(type_name, value);
        return {
          content: [
            {
              type: 'text',
              text: JSON.stringify({ hex: encoded.toHex(), type: type_name }, null, 2),
            },
          ],
        };
      } catch (err: any) {
        return {
          isError: true,
          content: [{ type: 'text', text: `Encode type error: ${err.message}` }],
        };
      }
    },
  );

  server.registerTool(
    'sails_decode_type',
    {
      description:
        'SCALE-decode a hex value for a specific IDL type. Returns the decoded JSON value.',
      inputSchema: {
        program: z.string().describe('Registered program name'),
        service: z.string().describe('Service name (for type resolution context)'),
        type_name: z.string().describe('Type name or SCALE type expression'),
        hex: z.string().describe('Hex-encoded value'),
      },
      annotations: { readOnlyHint: true },
    },
    async ({ program: programName, service: serviceName, type_name, hex }) => {
      try {
        const entry = registry.getOrThrow(programName);
        const services = entry.program.services;
        const service = services[serviceName];
        if (!service) {
          throw new Error(`Service "${serviceName}" not found`);
        }

        const decoded = service.registry.createType(type_name, hex as `0x${string}`);
        return {
          content: [
            {
              type: 'text',
              text: JSON.stringify(
                { value: decoded.toJSON(), type: type_name },
                null,
                2,
              ),
            },
          ],
        };
      } catch (err: any) {
        return {
          isError: true,
          content: [{ type: 'text', text: `Decode type error: ${err.message}` }],
        };
      }
    },
  );
}

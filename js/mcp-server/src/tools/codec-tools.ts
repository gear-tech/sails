import type { McpServer } from '@modelcontextprotocol/sdk/server/mcp.js';
import { z } from 'zod';
import { registry } from '../registry.js';

function getServiceFuncs(programName: string, serviceName: string) {
  const entry = registry.getOrThrow(programName);
  const services = entry.program.services;
  const service = services[serviceName];
  if (!service) {
    const available = Object.keys(services);
    throw new Error(
      `Service "${serviceName}" not found in program "${programName}". Available: [${available.join(', ')}]`,
    );
  }
  return service;
}

function getFunction(programName: string, serviceName: string, funcName: string) {
  const service = getServiceFuncs(programName, serviceName);
  // Search in both functions and queries
  const func = service.functions[funcName] ?? service.queries[funcName];
  if (!func) {
    const available = [...Object.keys(service.functions), ...Object.keys(service.queries)];
    throw new Error(
      `Function "${funcName}" not found in service "${serviceName}". Available: [${available.join(', ')}]`,
    );
  }
  return func;
}

export function registerCodecTools(server: McpServer) {
  server.registerTool(
    'sails_encode_payload',
    {
      description:
        'Encode a Sails function call to a hex string. Constructs the full message: 16-byte Sails Header v1 + SCALE-encoded arguments. ' +
        'The program must be parsed first via sails_parse_idl or sails_load_idl. ' +
        'Pass args as a JSON array matching the function parameter order. ' +
        'Type coercion: ActorId as "0x..." hex string, Option as null or value, enums as {"VariantName": value} or "UnitVariant".',
      inputSchema: {
        program: z.string().describe('Registered program name'),
        service: z.string().describe('Service name (e.g. "Counter")'),
        function: z.string().describe('Function name (e.g. "Add")'),
        args: z
          .array(z.any())
          .default([])
          .describe('Function arguments as JSON array, in parameter order'),
      },
      annotations: { readOnlyHint: true },
    },
    async ({ program: programName, service: serviceName, function: funcName, args }) => {
      try {
        const func = getFunction(programName, serviceName, funcName);
        const hex = func.encodePayload(...args);
        return {
          content: [
            {
              type: 'text',
              text: JSON.stringify(
                {
                  hex,
                  args_used: func.args.map((a, i) => ({
                    name: a.name,
                    type: a.type,
                    value: args[i] ?? null,
                  })),
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
          content: [{ type: 'text', text: `Encode error: ${error.message}` }],
        };
      }
    },
  );

  server.registerTool(
    'sails_decode_payload',
    {
      description:
        'Decode a hex payload back to structured arguments for a known function call. ' +
        'Parses the 16-byte Sails header and SCALE-decodes the remaining bytes. ' +
        'You must specify which program/service/function the payload belongs to.',
      inputSchema: {
        program: z.string().describe('Registered program name'),
        service: z.string().describe('Service name'),
        function: z.string().describe('Function name'),
        hex: z.string().describe('Hex-encoded payload (with or without 0x prefix)'),
      },
      annotations: { readOnlyHint: true },
    },
    async ({ program: programName, service: serviceName, function: funcName, hex }) => {
      try {
        const func = getFunction(programName, serviceName, funcName);
        const decoded = func.decodePayload(hex as `0x${string}`);
        return {
          content: [
            {
              type: 'text',
              text: JSON.stringify({ service: serviceName, function: funcName, args: decoded }, null, 2),
            },
          ],
        };
      } catch (error: any) {
        return {
          isError: true,
          content: [{ type: 'text', text: `Decode error: ${error.message}` }],
        };
      }
    },
  );

  server.registerTool(
    'sails_decode_result',
    {
      description:
        'Decode a function reply/result from hex. Strips the 16-byte Sails header and SCALE-decodes ' +
        'the return value according to the function return type from the IDL.',
      inputSchema: {
        program: z.string().describe('Registered program name'),
        service: z.string().describe('Service name'),
        function: z.string().describe('Function name'),
        hex: z.string().describe('Hex-encoded result payload'),
      },
      annotations: { readOnlyHint: true },
    },
    async ({ program: programName, service: serviceName, function: funcName, hex }) => {
      try {
        const func = getFunction(programName, serviceName, funcName);
        const result = func.decodeResult(hex as `0x${string}`);
        return {
          content: [
            {
              type: 'text',
              text: JSON.stringify({ service: serviceName, function: funcName, result }, null, 2),
            },
          ],
        };
      } catch (error: any) {
        return {
          isError: true,
          content: [{ type: 'text', text: `Decode error: ${error.message}` }],
        };
      }
    },
  );

  server.registerTool(
    'sails_encode_constructor',
    {
      description:
        'Encode a program constructor call to hex. Used for program initialization (upload_program / create_program).',
      inputSchema: {
        program: z.string().describe('Registered program name'),
        // Avoid the field name "constructor": MCP arguments are validated as a record,
        // and that key is unsafe in the current SDK stack.
        ctor: z.string().describe('Constructor name (e.g. "New", "Default")'),
        args: z.array(z.any()).default([]).describe('Constructor arguments as JSON array'),
      },
      annotations: { readOnlyHint: true },
    },
    async ({ program: programName, ctor: ctorName, args }) => {
      try {
        const entry = registry.getOrThrow(programName);
        const ctors = entry.program.ctors;
        if (!ctors) {
          throw new Error(`Program "${programName}" has no constructors`);
        }
        const ctor = ctors[ctorName];
        if (!ctor) {
          const available = Object.keys(ctors);
          throw new Error(
            `Constructor "${ctorName}" not found. Available: [${available.join(', ')}]`,
          );
        }
        const hex = ctor.encodePayload(...args);
        return {
          content: [{ type: 'text', text: JSON.stringify({ hex }, null, 2) }],
        };
      } catch (error: any) {
        return {
          isError: true,
          content: [{ type: 'text', text: `Encode error: ${error.message}` }],
        };
      }
    },
  );

  server.registerTool(
    'sails_decode_event',
    {
      description:
        'Decode an event payload from hex. If event name is omitted, auto-detects by matching ' +
        "the header's interface_id + entry_id against all events in the service. " +
        'Returns the event name and decoded field values.',
      inputSchema: {
        program: z.string().describe('Registered program name'),
        service: z.string().describe('Service name'),
        hex: z.string().describe('Hex-encoded event payload'),
        event: z
          .string()
          .optional()
          .describe('Event name. If omitted, auto-detects from header entry_id.'),
      },
      annotations: { readOnlyHint: true },
    },
    async ({ program: programName, service: serviceName, hex, event: eventName }) => {
      try {
        const service = getServiceFuncs(programName, serviceName);
        const events = service.events;

        if (eventName) {
          const ev = events[eventName];
          if (!ev) {
            const available = Object.keys(events);
            throw new Error(
              `Event "${eventName}" not found in service "${serviceName}". Available: [${available.join(', ')}]`,
            );
          }
          const data = ev.decode(hex as `0x${string}`);
          return {
            content: [
              {
                type: 'text',
                text: JSON.stringify({ event: eventName, data }, null, 2),
              },
            ],
          };
        }

        // Auto-detect: try each event's decode
        for (const [name, ev] of Object.entries(events)) {
          try {
            const data = ev.decode(hex as `0x${string}`);
            return {
              content: [
                {
                  type: 'text',
                  text: JSON.stringify({ event: name, data, auto_detected: true }, null, 2),
                },
              ],
            };
          } catch {
            continue;
          }
        }

        throw new Error(
          `Could not match event in service "${serviceName}". No event entry_id matched the payload header.`,
        );
      } catch (error: any) {
        return {
          isError: true,
          content: [{ type: 'text', text: `Event decode error: ${error.message}` }],
        };
      }
    },
  );
}

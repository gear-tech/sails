import type { McpServer } from '@modelcontextprotocol/sdk/server/mcp.js';
import { z } from 'zod';
import {
  encodeAbiParameters,
  decodeAbiParameters,
  keccak256,
  toHex,
  type AbiParameter,
} from 'viem';
import { registry } from '../registry.js';
import type { TypeDecl, IIdlDoc, IServiceUnit, IServiceFunc, IStructField } from 'sails-js-types';

// ---------------------------------------------------------------------------
// Type mapping: IDL TypeDecl → Solidity type string
// Based on rs/sol-gen/src/typedecl_to_sol.rs
// ---------------------------------------------------------------------------

export function typeDeclToSolidity(td: TypeDecl): string {
  if (typeof td === 'string') {
    switch (td) {
      case 'bool': return 'bool';
      case 'u8': return 'uint8';
      case 'u16': return 'uint16';
      case 'u32': return 'uint32';
      case 'u64': return 'uint64';
      case 'u128': return 'uint128';
      case 'U256': return 'uint256';
      case 'i8': return 'int8';
      case 'i16': return 'int16';
      case 'i32': return 'int32';
      case 'i64': return 'int64';
      case 'i128': return 'int128';
      case 'String': return 'string';
      case 'ActorId': return 'address';
      case 'H256':
      case 'CodeId':
      case 'MessageId': return 'bytes32';
      case 'H160': return 'bytes20';
      case '()': throw new Error('Void type () has no Solidity representation');
      case 'char': throw new Error('char type is not supported in Solidity ABI');
      default: throw new Error(`Unsupported primitive type in ABI: ${td}`);
    }
  }
  if (td.kind === 'slice') {
    return `${typeDeclToSolidity(td.item)}[]`;
  }
  if (td.kind === 'array') {
    return `${typeDeclToSolidity(td.item)}[${td.len}]`;
  }
  if (td.kind === 'tuple') {
    throw new Error('Tuple types are not supported in Sails ABI encoding');
  }
  if (td.kind === 'named') {
    throw new Error(
      `Named type "${td.name}" is not supported in Sails ABI encoding. ` +
        'Only primitives, slices, and fixed arrays are allowed.',
    );
  }
  throw new Error(`Unknown TypeDecl: ${JSON.stringify(td)}`);
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

function toAbiParam(name: string, type: TypeDecl): AbiParameter {
  return { name, type: typeDeclToSolidity(type) } as AbiParameter;
}

/** camelCase(serviceName + funcName): "Counter" + "Add" → "counterAdd" */
function solFuncName(serviceName: string, funcName: string): string {
  const combined = serviceName + funcName;
  return combined[0].toLowerCase() + combined.slice(1);
}

/** Build a Solidity-style signature string, e.g. "counterAdd(bool,uint32)" */
function buildSignature(name: string, paramTypes: string[]): string {
  return `${name}(${paramTypes.join(',')})`;
}

/** Compute the 4-byte Solidity selector from a signature string */
function computeSelector(sig: string): `0x${string}` {
  const hash = keccak256(toHex(sig));
  return hash.slice(0, 10) as `0x${string}`;
}

function getServiceUnit(entry: ReturnType<typeof registry.getOrThrow>, serviceName: string): IServiceUnit {
  const unit = entry.doc.services?.find((s) => s.name === serviceName);
  if (!unit) {
    const available = entry.doc.services?.map((s) => s.name) ?? [];
    throw new Error(`Service "${serviceName}" not found. Available: [${available.join(', ')}]`);
  }
  return unit;
}

function getIdlFunc(
  entry: ReturnType<typeof registry.getOrThrow>,
  serviceName: string,
  funcName: string,
): IServiceFunc {
  const unit = getServiceUnit(entry, serviceName);
  const func = unit.funcs?.find((f) => f.name === funcName);
  if (!func) {
    const available = unit.funcs?.map((f) => f.name) ?? [];
    throw new Error(
      `Function "${funcName}" not found in service "${serviceName}". Available: [${available.join(', ')}]`,
    );
  }
  return func;
}

function isPayable(func: IServiceFunc): boolean {
  return func.docs?.some((d) => d.includes('#[payable]')) ?? false;
}

function isReturnsValue(func: IServiceFunc): boolean {
  return func.docs?.some((d) => d.includes('#[returns_value]')) ?? false;
}

function isIndexed(field: IStructField): boolean {
  return field.docs?.some((d) => d.includes('#[indexed]')) ?? false;
}

// ---------------------------------------------------------------------------
// Solidity contract generation
// Based on rs/sol-gen/src/hbs/contract.hbs and rs/sol-gen/src/lib.rs
// ---------------------------------------------------------------------------

function generateSolidity(doc: IIdlDoc, baseName: string): string {
  const services = doc.services ?? [];
  const lines: string[] = [];

  lines.push('// SPDX-License-Identifier: UNLICENSED');
  lines.push('pragma solidity ^0.8.26;');
  lines.push('');

  // Contract 1: Interface
  lines.push(`interface I${baseName} {`);
  for (const unit of services) {
    for (const func of unit.funcs ?? []) {
      const solName = solFuncName(unit.name, func.name);
      const payableKw = isPayable(func) ? ' payable' : '';
      const paramList = [
        'bool _callReply',
        ...(func.params ?? []).map((p) => `${typeDeclToSolidity(p.type)} ${p.name}`),
      ].join(', ');
      const hasOutput = func.output && func.output !== '()';
      const returnType = hasOutput ? typeDeclToSolidity(func.output) : null;

      lines.push(
        `    function ${solName}(${paramList}) external${payableKw} returns (bytes32 messageId);`,
      );
      // Additional overload for #[returns_value]
      if (isReturnsValue(func) && returnType) {
        lines.push(
          `    function ${solName}(${paramList}, bool returns_value) external${payableKw} returns (bytes32 messageId, ${returnType} result);`,
        );
      }
    }
    // Events
    for (const event of unit.events ?? []) {
      const fieldList = (event.fields ?? [])
        .map((f, i) => {
          const solType = typeDeclToSolidity(f.type);
          const indexed = isIndexed(f) ? ' indexed' : '';
          return `${solType}${indexed} ${f.name ?? `field${i}`}`;
        })
        .join(', ');
      lines.push(`    event ${event.name}(${fieldList});`);
    }
  }
  lines.push('}');
  lines.push('');

  // Contract 2: Abi (pure selector functions)
  lines.push(`abstract contract ${baseName}Abi {`);
  for (const unit of services) {
    for (const func of unit.funcs ?? []) {
      const solName = solFuncName(unit.name, func.name);
      const paramTypes = [
        'bool',
        ...(func.params ?? []).map((p) => typeDeclToSolidity(p.type)),
      ];
      const sig = buildSignature(solName, paramTypes);
      lines.push(`    function ${solName}_sig() public pure returns (bytes4) {`);
      lines.push(`        return bytes4(keccak256("${sig}"));`);
      lines.push(`    }`);
    }
  }
  lines.push('}');
  lines.push('');

  // Contract 3: Callbacks interface
  lines.push(`interface I${baseName}Callbacks {`);
  for (const unit of services) {
    for (const func of unit.funcs ?? []) {
      if (func.output && func.output !== '()') {
        const solName = solFuncName(unit.name, func.name);
        const returnType = typeDeclToSolidity(func.output);
        lines.push(`    function replyOn_${solName}(${returnType} result) external;`);
      }
    }
  }
  lines.push('}');
  lines.push('');

  // Contract 4: Caller (abstract helper)
  lines.push(`abstract contract ${baseName}Caller {`);
  lines.push(`    address public immutable program;`);
  lines.push(`    constructor(address _program) { program = _program; }`);
  lines.push('');
  for (const unit of services) {
    for (const func of unit.funcs ?? []) {
      const solName = solFuncName(unit.name, func.name);
      const payableKw = isPayable(func) ? ' payable' : '';
      const paramList = (func.params ?? [])
        .map((p) => `${typeDeclToSolidity(p.type)} ${p.name}`)
        .join(', ');
      const passArgs = (func.params ?? []).map((p) => p.name).join(', ');
      const callArgs = passArgs ? `false, ${passArgs}` : 'false';
      lines.push(
        `    function ${solName}(${paramList}) internal${payableKw} returns (bytes32 messageId) {`,
      );
      lines.push(`        return I${baseName}(program).${solName}(${callArgs});`);
      lines.push(`    }`);
    }
  }
  lines.push('}');

  return lines.join('\n');
}

// ---------------------------------------------------------------------------
// Tool registration
// ---------------------------------------------------------------------------

export function registerAbiTools(server: McpServer) {
  server.registerTool(
    'sails_type_to_solidity',
    {
      description:
        'Map an IDL type to its Solidity equivalent. ' +
        'Supported: bool, u8-u128, U256, i8-i128, String→string, ActorId→address, ' +
        'H256/CodeId/MessageId→bytes32, H160→bytes20, slices→T[], fixed arrays→T[N]. ' +
        'Pass a primitive name (e.g. "u32") or a JSON TypeDecl object.',
      inputSchema: {
        type_decl: z
          .string()
          .describe('IDL type: a primitive name like "u32" or JSON like {"kind":"slice","item":"u8"}'),
      },
      annotations: { readOnlyHint: true },
    },
    async ({ type_decl }) => {
      try {
        let td: TypeDecl;
        try {
          td = JSON.parse(type_decl) as TypeDecl;
        } catch {
          td = type_decl as TypeDecl;
        }
        const solType = typeDeclToSolidity(td);
        return {
          content: [
            {
              type: 'text',
              text: JSON.stringify({ idl_type: type_decl, solidity_type: solType }, null, 2),
            },
          ],
        };
      } catch (err: any) {
        return { isError: true, content: [{ type: 'text', text: err.message }] };
      }
    },
  );

  server.registerTool(
    'sails_solidity_signature',
    {
      description:
        'Compute the Solidity function/event signature and 4-byte selector for an IDL function or event. ' +
        'Functions include the implicit bool _callReply first param. ' +
        'Function names follow ethexe convention: camelCase(ServiceName + FunctionName).',
      inputSchema: {
        program: z.string().describe('Registered program name'),
        service: z.string().describe('Service name'),
        name: z.string().describe('Function or event name'),
        kind: z
          .enum(['function', 'event'])
          .default('function')
          .describe('Whether to compute selector for a function or event'),
      },
      annotations: { readOnlyHint: true },
    },
    async ({ program: programName, service: serviceName, name: targetName, kind }) => {
      try {
        const entry = registry.getOrThrow(programName);
        const unit = getServiceUnit(entry, serviceName);

        if (kind === 'function') {
          const func = unit.funcs?.find((f) => f.name === targetName);
          if (!func) {
            throw new Error(`Function "${targetName}" not found in service "${serviceName}"`);
          }
          const solName = solFuncName(serviceName, targetName);
          const paramTypes = [
            'bool',
            ...(func.params ?? []).map((p) => typeDeclToSolidity(p.type)),
          ];
          const sig = buildSignature(solName, paramTypes);
          const sel = computeSelector(sig);
          return {
            content: [
              {
                type: 'text',
                text: JSON.stringify(
                  { signature: sig, selector: sel, solidity_name: solName, payable: isPayable(func) },
                  null,
                  2,
                ),
              },
            ],
          };
        } else {
          const event = unit.events?.find((e) => e.name === targetName);
          if (!event) {
            throw new Error(`Event "${targetName}" not found in service "${serviceName}"`);
          }
          const fieldTypes = (event.fields ?? []).map((f) => typeDeclToSolidity(f.type));
          const sig = buildSignature(targetName, fieldTypes);
          const sel = computeSelector(sig);
          return {
            content: [
              {
                type: 'text',
                text: JSON.stringify({ signature: sig, selector: sel }, null, 2),
              },
            ],
          };
        }
      } catch (err: any) {
        return { isError: true, content: [{ type: 'text', text: err.message }] };
      }
    },
  );

  server.registerTool(
    'sails_abi_encode_call',
    {
      description:
        'ABI-encode an ethexe function call: 4-byte selector + ABI-encoded params. ' +
        'Automatically prepends the implicit bool _callReply first param. ' +
        'Function selector uses camelCase(ServiceName+FunctionName) naming. ' +
        'For u64/u128/U256 values pass numeric strings to avoid JS integer overflow.',
      inputSchema: {
        program: z.string().describe('Registered program name'),
        service: z.string().describe('Service name'),
        function: z.string().describe('Function name'),
        args: z
          .array(z.any())
          .default([])
          .describe('Function args as JSON array in parameter order (excluding _callReply)'),
        call_reply: z
          .boolean()
          .default(false)
          .describe('Value for the implicit bool _callReply first param'),
      },
      annotations: { readOnlyHint: true },
    },
    async ({ program: programName, service: serviceName, function: funcName, args, call_reply }) => {
      try {
        const entry = registry.getOrThrow(programName);
        const func = getIdlFunc(entry, serviceName, funcName);
        const solName = solFuncName(serviceName, funcName);
        const paramTypes = [
          'bool',
          ...(func.params ?? []).map((p) => typeDeclToSolidity(p.type)),
        ];
        const sig = buildSignature(solName, paramTypes);
        const sel = computeSelector(sig);

        const abiParams: AbiParameter[] = [
          { name: '_callReply', type: 'bool' },
          ...(func.params ?? []).map((p) => toAbiParam(p.name, p.type)),
        ];
        const encoded = encodeAbiParameters(abiParams, [call_reply, ...args]);
        const hex = (sel + encoded.slice(2)) as `0x${string}`;

        return {
          content: [
            {
              type: 'text',
              text: JSON.stringify(
                {
                  hex,
                  selector: sel,
                  signature: sig,
                  payable: isPayable(func),
                  explanation: `${sel} (selector) + ABI-encoded(_callReply=${call_reply}, ${(func.params ?? [])
                    .map((p, i) => `${p.name}=${JSON.stringify(args[i] ?? null)}`)
                    .join(', ')})`,
                },
                null,
                2,
              ),
            },
          ],
        };
      } catch (err: any) {
        return {
          isError: true,
          content: [{ type: 'text', text: `ABI encode error: ${err.message}` }],
        };
      }
    },
  );

  server.registerTool(
    'sails_abi_decode_call',
    {
      description:
        'ABI-decode a hex payload by matching the first 4 bytes (selector) against all known functions in a program. ' +
        'Returns the matched service/function name and decoded arguments.',
      inputSchema: {
        program: z.string().describe('Registered program name'),
        hex: z.string().describe('Hex-encoded payload (4-byte selector + ABI-encoded params)'),
      },
      annotations: { readOnlyHint: true },
    },
    async ({ program: programName, hex: hexInput }) => {
      try {
        const entry = registry.getOrThrow(programName);
        const normalized = hexInput.startsWith('0x') ? hexInput : `0x${hexInput}`;
        const callSelector = normalized.slice(0, 10).toLowerCase();

        for (const unit of entry.doc.services ?? []) {
          for (const func of unit.funcs ?? []) {
            let paramTypes: string[];
            try {
              paramTypes = [
                'bool',
                ...(func.params ?? []).map((p) => typeDeclToSolidity(p.type)),
              ];
            } catch {
              continue; // skip functions with unsupported types
            }
            const sig = buildSignature(solFuncName(unit.name, func.name), paramTypes);
            const sel = computeSelector(sig);

            if (sel.toLowerCase() === callSelector) {
              const abiParams: AbiParameter[] = [
                { name: '_callReply', type: 'bool' },
                ...(func.params ?? []).map((p) => toAbiParam(p.name, p.type)),
              ];
              const decoded = decodeAbiParameters(
                abiParams,
                `0x${normalized.slice(10)}` as `0x${string}`,
              );
              const [callReply, ...funcArgs] = decoded;
              const argsObj: Record<string, unknown> = {};
              (func.params ?? []).forEach((p, i) => {
                argsObj[p.name] = funcArgs[i];
              });

              return {
                content: [
                  {
                    type: 'text',
                    text: JSON.stringify(
                      {
                        service: unit.name,
                        function: func.name,
                        selector: sel,
                        _callReply: callReply,
                        args: argsObj,
                      },
                      null,
                      2,
                    ),
                  },
                ],
              };
            }
          }
        }
        throw new Error(
          `No function matches selector ${callSelector} in program "${programName}"`,
        );
      } catch (err: any) {
        return {
          isError: true,
          content: [{ type: 'text', text: `ABI decode error: ${err.message}` }],
        };
      }
    },
  );

  server.registerTool(
    'sails_abi_decode_result',
    {
      description:
        'ABI-decode a function reply from hex. ' +
        'Decodes the return value according to the IDL function output type.',
      inputSchema: {
        program: z.string().describe('Registered program name'),
        service: z.string().describe('Service name'),
        function: z.string().describe('Function name'),
        hex: z.string().describe('Hex-encoded ABI result payload'),
      },
      annotations: { readOnlyHint: true },
    },
    async ({ program: programName, service: serviceName, function: funcName, hex: hexInput }) => {
      try {
        const entry = registry.getOrThrow(programName);
        const func = getIdlFunc(entry, serviceName, funcName);

        if (!func.output || func.output === '()') {
          return {
            content: [
              {
                type: 'text',
                text: JSON.stringify(
                  { result: null, note: 'Function returns void' },
                  null,
                  2,
                ),
              },
            ],
          };
        }

        const returnType = typeDeclToSolidity(func.output);
        const normalized = (
          hexInput.startsWith('0x') ? hexInput : `0x${hexInput}`
        ) as `0x${string}`;
        const decoded = decodeAbiParameters([{ name: 'result', type: returnType }], normalized);

        return {
          content: [
            {
              type: 'text',
              text: JSON.stringify(
                { service: serviceName, function: funcName, result: decoded[0] },
                null,
                2,
              ),
            },
          ],
        };
      } catch (err: any) {
        return {
          isError: true,
          content: [{ type: 'text', text: `ABI decode error: ${err.message}` }],
        };
      }
    },
  );

  server.registerTool(
    'sails_abi_encode_event',
    {
      description:
        'ABI-encode an ethexe event into topics and data. ' +
        'topics[0] = keccak256 event signature hash. ' +
        'Fields marked #[indexed] in IDL become additional topics; non-indexed fields are ABI-encoded as data.',
      inputSchema: {
        program: z.string().describe('Registered program name'),
        service: z.string().describe('Service name'),
        event: z.string().describe('Event name'),
        fields: z
          .record(z.any())
          .describe('Event field values as a JSON object { fieldName: value }'),
      },
      annotations: { readOnlyHint: true },
    },
    async ({ program: programName, service: serviceName, event: eventName, fields }) => {
      try {
        const entry = registry.getOrThrow(programName);
        const unit = getServiceUnit(entry, serviceName);
        const event = unit.events?.find((e) => e.name === eventName);
        if (!event) {
          throw new Error(`Event "${eventName}" not found in service "${serviceName}"`);
        }

        const allFields = event.fields ?? [];
        const indexedFields = allFields.filter(isIndexed);
        const dataFields = allFields.filter((f) => !isIndexed(f));

        // Event signature hash → topic[0]
        const fieldTypes = allFields.map((f) => typeDeclToSolidity(f.type));
        const sig = buildSignature(eventName, fieldTypes);
        const eventTopic = keccak256(toHex(sig));

        // Indexed fields → additional topics (ABI-encode each as 32 bytes)
        const topics: string[] = [eventTopic];
        for (const field of indexedFields) {
          const solType = typeDeclToSolidity(field.type);
          const val = fields[field.name ?? ''];
          const encoded = encodeAbiParameters([{ name: field.name ?? '', type: solType }], [val]);
          // Take the last 32 bytes (64 hex chars) as topic
          topics.push(`0x${encoded.slice(2).slice(-64).padStart(64, '0')}`);
        }

        // Non-indexed fields → data
        let data: string = '0x';
        if (dataFields.length > 0) {
          const dataParams: AbiParameter[] = dataFields.map((f) =>
            toAbiParam(f.name ?? '', f.type),
          );
          const dataValues = dataFields.map((f) => fields[f.name ?? '']);
          data = encodeAbiParameters(dataParams, dataValues);
        }

        return {
          content: [
            {
              type: 'text',
              text: JSON.stringify({ event: eventName, signature: sig, topics, data }, null, 2),
            },
          ],
        };
      } catch (err: any) {
        return {
          isError: true,
          content: [{ type: 'text', text: `ABI event encode error: ${err.message}` }],
        };
      }
    },
  );

  server.registerTool(
    'sails_abi_decode_event',
    {
      description:
        'ABI-decode an ethexe event from topics and data. ' +
        'Matches topics[0] (event signature hash) to identify the event if event name is omitted.',
      inputSchema: {
        program: z.string().describe('Registered program name'),
        service: z.string().describe('Service name'),
        topics: z
          .array(z.string())
          .describe('Hex topic strings; topics[0] is the event signature hash'),
        data: z.string().default('0x').describe('Hex-encoded ABI data for non-indexed fields'),
        event: z
          .string()
          .optional()
          .describe('Event name. If omitted, auto-detects from topics[0].'),
      },
      annotations: { readOnlyHint: true },
    },
    async ({ program: programName, service: serviceName, topics, data, event: eventName }) => {
      try {
        const entry = registry.getOrThrow(programName);
        const unit = getServiceUnit(entry, serviceName);

        let targetEvent = unit.events?.find((e) => e.name === eventName);
        if (!targetEvent && eventName) {
          throw new Error(`Event "${eventName}" not found in service "${serviceName}"`);
        }

        if (!targetEvent) {
          const topic0 = (topics[0] ?? '').toLowerCase();
          for (const ev of unit.events ?? []) {
            const fieldTypes = (ev.fields ?? []).map((f) => typeDeclToSolidity(f.type));
            const sig = buildSignature(ev.name, fieldTypes);
            if (keccak256(toHex(sig)).toLowerCase() === topic0) {
              targetEvent = ev;
              break;
            }
          }
          if (!targetEvent) {
            throw new Error(
              `No event in service "${serviceName}" matches topics[0]=${topics[0]}`,
            );
          }
        }

        const allFields = targetEvent.fields ?? [];
        const indexedFields = allFields.filter(isIndexed);
        const dataFields = allFields.filter((f) => !isIndexed(f));
        const result: Record<string, unknown> = {};

        // Decode indexed fields from topics[1..]
        indexedFields.forEach((field, i) => {
          const topic = topics[i + 1];
          if (topic) {
            const solType = typeDeclToSolidity(field.type);
            const padded = topic.startsWith('0x') ? topic : `0x${topic}`;
            // Pad to 32 bytes for decoding
            const padded32 = `0x${padded.slice(2).padStart(64, '0')}` as `0x${string}`;
            const decoded = decodeAbiParameters(
              [{ name: field.name ?? '', type: solType }],
              padded32,
            );
            result[field.name ?? `indexed_${i}`] = decoded[0];
          }
        });

        // Decode non-indexed from data
        if (dataFields.length > 0 && data && data !== '0x') {
          const dataParams: AbiParameter[] = dataFields.map((f) =>
            toAbiParam(f.name ?? '', f.type),
          );
          const decoded = decodeAbiParameters(dataParams, data as `0x${string}`);
          dataFields.forEach((f, i) => {
            result[f.name ?? `data_${i}`] = decoded[i];
          });
        }

        return {
          content: [
            {
              type: 'text',
              text: JSON.stringify({ event: targetEvent.name, fields: result }, null, 2),
            },
          ],
        };
      } catch (err: any) {
        return {
          isError: true,
          content: [{ type: 'text', text: `ABI event decode error: ${err.message}` }],
        };
      }
    },
  );

  server.registerTool(
    'sails_generate_solidity',
    {
      description:
        'Generate a Solidity interface from a registered IDL program. ' +
        'Produces 4 contracts: I{Name} (interface), {Name}Abi (selectors), ' +
        'I{Name}Callbacks (reply callbacks), {Name}Caller (abstract caller). ' +
        'Ethexe-compatible: functions have implicit bool _callReply first param, ' +
        'payable detection from #[payable], returns_value from #[returns_value].',
      inputSchema: {
        program: z.string().describe('Registered program name'),
        contract_name: z
          .string()
          .optional()
          .describe('Base name for generated contracts (defaults to program name)'),
      },
      annotations: { readOnlyHint: true },
    },
    async ({ program: programName, contract_name }) => {
      try {
        const entry = registry.getOrThrow(programName);
        const baseName = contract_name ?? entry.doc.program?.name ?? programName;
        const solidity = generateSolidity(entry.doc, baseName);
        return {
          content: [{ type: 'text', text: solidity }],
        };
      } catch (err: any) {
        return {
          isError: true,
          content: [{ type: 'text', text: `Solidity generation error: ${err.message}` }],
        };
      }
    },
  );
}

#!/usr/bin/env bun
import { McpServer } from '@modelcontextprotocol/sdk/server/mcp.js';
import { StdioServerTransport } from '@modelcontextprotocol/sdk/server/stdio.js';

import { registerIdlTools, getParser } from './tools/idl-tools.js';
import { registerSourceTools } from './tools/source-tools.js';
import { registerCodecTools } from './tools/codec-tools.js';
import { registerHeaderTools } from './tools/header-tools.js';
import { registerTypeTools } from './tools/type-tools.js';
import { registerUtilTools } from './tools/util-tools.js';
import { registerResources } from './resources.js';

const server = new McpServer(
  {
    name: 'sails-mcp',
    version: '0.5.1',
  },
  {
    instructions:
      'Sails IDL & Program development server. Start by parsing an IDL with sails_parse_idl ' +
      'or loading from file with sails_load_idl. Then use encoding/decoding tools to work with payloads. ' +
      'All encode/decode tools require a program to be registered first. ' +
      'Use sails_list_programs to see registered programs.',
  },
);

// Register all tool categories
registerIdlTools(server);
registerSourceTools(server);
registerCodecTools(server);
registerHeaderTools(server);
registerTypeTools(server);
registerUtilTools(server);
registerResources(server);

// Initialize the WASM parser eagerly so first tool call is fast
getParser().catch((err) => {
  console.error('Warning: Failed to pre-initialize IDL parser:', err.message);
});

// Start the stdio transport
const transport = new StdioServerTransport();
await server.connect(transport);

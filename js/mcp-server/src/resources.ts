import type { McpServer } from '@modelcontextprotocol/sdk/server/mcp.js';
import { readFile } from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

// Resolve paths relative to the repo root (js/mcp-server -> repo root)
function repoRoot(): string {
  const thisDir = path.dirname(fileURLToPath(import.meta.url));
  return path.resolve(thisDir, '..', '..', '..');
}

async function readRepoFile(relativePath: string): Promise<string> {
  const fullPath = path.resolve(repoRoot(), relativePath);
  return readFile(fullPath, 'utf8');
}

export function registerResources(server: McpServer) {
  server.registerResource(
    'sails://specs/header-v1',
    'sails://specs/header-v1',
    {
      description: 'Sails Message Header v1 specification — 16-byte binary header format for routing messages.',
      mimeType: 'text/markdown',
    },
    async () => {
      const content = await readRepoFile('docs/sails-header-v1-spec.md');
      return { contents: [{ uri: 'sails://specs/header-v1', text: content, mimeType: 'text/markdown' }] };
    },
  );

  server.registerResource(
    'sails://specs/idl-v2',
    'sails://specs/idl-v2',
    {
      description: 'Sails IDL v2 language specification — grammar, types, services, programs.',
      mimeType: 'text/markdown',
    },
    async () => {
      const content = await readRepoFile('docs/idl-v2-spec.md');
      return { contents: [{ uri: 'sails://specs/idl-v2', text: content, mimeType: 'text/markdown' }] };
    },
  );

  server.registerResource(
    'sails://specs/interface-id',
    'sails://specs/interface-id',
    {
      description: 'Interface ID specification — how structural BLAKE3 hashes identify services.',
      mimeType: 'text/markdown',
    },
    async () => {
      const spec = await readRepoFile('docs/interface-id-spec.md');
      const hash = await readRepoFile('docs/reflect-hash-spec.md');
      return {
        contents: [
          { uri: 'sails://specs/interface-id', text: `${spec}\n\n---\n\n${hash}`, mimeType: 'text/markdown' },
        ],
      };
    },
  );

  server.registerResource(
    'sails://examples/demo',
    'sails://examples/demo',
    {
      description: 'Complete demo IDL file showing services, events, types, constructors, and inheritance.',
      mimeType: 'text/plain',
    },
    async () => {
      const content = await readRepoFile('examples/demo/client/demo_client.idl');
      return { contents: [{ uri: 'sails://examples/demo', text: content, mimeType: 'text/plain' }] };
    },
  );
}

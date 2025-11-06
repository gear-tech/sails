import { readFile } from 'node:fs/promises';
import { fileURLToPath } from 'node:url';
import path from 'node:path';

import canonicalize from 'canonicalize';
import { createHash } from 'blake3';

const DOMAIN = 'SAILS-IDL/v1/interface-id';
const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const canonicalDir = path.resolve(__dirname, '../canonical');

const manifestPath = path.join(canonicalDir, 'manifest.json');
const manifestRaw = await readFile(manifestPath, 'utf8');
const manifest = JSON.parse(manifestRaw);

let failures = 0;
for (const [file, services] of Object.entries(manifest)) {
  const docPath = path.join(canonicalDir, file);
  const jsonStr = await readFile(docPath, 'utf8');
  const document = JSON.parse(jsonStr);

  for (const [serviceName, expectedHex] of Object.entries(services)) {
    const service = document.services?.[serviceName];
    if (!service) {
      console.error(`Service ${serviceName} not found in ${file}`);
      failures += 1;
      continue;
    }

    const singleDoc = {
      canon_schema: document.canon_schema ?? 'sails-idl-jcs',
      canon_version: document.canon_version ?? document.version ?? '1',
      hash: document.hash ?? { algo: 'blake3', domain: DOMAIN },
      services: {
        [serviceName]: service,
      },
      types: document.types ?? {},
    };

    const canonicalJson = canonicalize(singleDoc);
    const hasher = createHash();
    hasher.update(Buffer.from(DOMAIN, 'utf8'));
    hasher.update(Buffer.from(canonicalJson, 'utf8'));
    const digest = hasher.digest();
    const actualId = digest.readBigUInt64LE(0);

    const expectedId = BigInt(expectedHex);
    console.log(expectedId, actualId);
    
    if (actualId !== expectedId) {
      console.error(
        `Mismatch for ${serviceName} in ${file}: expected ${expectedHex}, got 0x${actualId.toString(16).padStart(16, '0')}`,
      );
      failures += 1;
    }
  }
}

if (failures > 0) {
  process.exitCode = 1;
} else {
  console.log('Canonical vectors verified successfully.');
}

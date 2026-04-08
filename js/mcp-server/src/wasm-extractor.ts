import { readFile } from 'node:fs/promises';
import { inflate } from 'node:zlib';
import { promisify } from 'node:util';

const inflateAsync = promisify(inflate);

const SECTION_NAME = 'sails:idl';
const ENVELOPE_VERSION = 0x01;
const FLAG_COMPRESSED = 0x01;
const MAX_DECOMPRESSED_SIZE = 1024 * 1024; // 1 MB

/**
 * Extract IDL text from a WASM binary's "sails:idl" custom section.
 *
 * Mirrors the logic in rs/idl-embed/src/lib.rs.
 *
 * WASM custom sections have:
 * - section id = 0
 * - name string (LEB128 length + UTF-8 name)
 * - raw bytes (the payload)
 *
 * The envelope format is:
 * - byte 0: version (must be 0x01)
 * - byte 1: flags (0x01 = deflate compressed)
 * - bytes 2+: content (raw or deflated IDL text)
 */
export async function extractIdlFromWasm(wasmPath: string): Promise<string | null> {
  const wasmBytes = await readFile(wasmPath);
  return extractIdlFromBytes(wasmBytes);
}

export function extractIdlFromBytes(wasmBytes: Buffer | Uint8Array): Promise<string | null> {
  return parseWasmCustomSection(wasmBytes, SECTION_NAME);
}

async function parseWasmCustomSection(
  wasm: Buffer | Uint8Array,
  sectionName: string,
): Promise<string | null> {
  const view = new DataView(wasm.buffer, wasm.byteOffset, wasm.byteLength);
  let offset = 0;

  // WASM magic: \0asm
  if (wasm.length < 8) return null;
  if (view.getUint32(0, true) !== 0x6D_73_61_00) return null; // \0asm in LE
  offset = 4;

  // WASM version
  const version = view.getUint32(offset, true);
  if (version !== 1) return null;
  offset += 4;

  // Iterate sections
  while (offset < wasm.length) {
    const sectionId = wasm[offset];
    offset += 1;

    // Read section size (LEB128)
    const { value: sectionSize, bytesRead: sizeBytes } = readLeb128(wasm, offset);
    offset += sizeBytes;

    const sectionEnd = offset + sectionSize;

    if (sectionId === 0) {
      // Custom section - read name
      const { value: nameLen, bytesRead: nameLenBytes } = readLeb128(wasm, offset);
      const nameStart = offset + nameLenBytes;
      const nameBytes = wasm.slice(nameStart, nameStart + nameLen);
      const name = new TextDecoder().decode(nameBytes);

      if (name === sectionName) {
        const dataStart = nameStart + nameLen;
        const data = wasm.slice(dataStart, sectionEnd);
        return decodeEnvelope(data);
      }
    }

    offset = sectionEnd;
  }

  return null;
}

async function decodeEnvelope(data: Uint8Array): Promise<string | null> {
  if (data.length < 2) return null;

  const version = data[0];
  if (version !== ENVELOPE_VERSION) return null;

  const flags = data[1];
  // Unknown flags - skip gracefully
  if ((flags & ~FLAG_COMPRESSED) !== 0) return null;

  const content = data.slice(2);

  if (flags & FLAG_COMPRESSED) {
    try {
      const decompressed = await inflateAsync(content, { maxOutputLength: MAX_DECOMPRESSED_SIZE });
      return new TextDecoder().decode(decompressed);
    } catch {
      throw new Error('Failed to decompress IDL from WASM section');
    }
  } else {
    if (content.length > MAX_DECOMPRESSED_SIZE) {
      throw new Error('IDL content exceeds maximum size (1MB)');
    }
    return new TextDecoder().decode(content);
  }
}

function readLeb128(bytes: Uint8Array, offset: number): { value: number; bytesRead: number } {
  let result = 0;
  let shift = 0;
  let bytesRead = 0;

  while (offset + bytesRead < bytes.length) {
    const byte = bytes[offset + bytesRead];
    result |= (byte & 0x7F) << shift;
    bytesRead++;
    if ((byte & 0x80) === 0) break;
    shift += 7;
  }

  return { value: result, bytesRead };
}

const WASM_MAGIC = new Uint8Array([0x00, 0x61, 0x73, 0x6D]);
const WASM_VERSION = new Uint8Array([0x01, 0x00, 0x00, 0x00]);
const SECTION_CUSTOM = 0;
const SECTION_NAME = 'sails:idl';
const ENVELOPE_VERSION = 0x01;
const FLAG_COMPRESSED = 0x01;
const MAX_DECOMPRESSED_SIZE = 1024 * 1024;

export class WasmParseError extends Error {
  constructor(message: string) {
    super(message);
    this.name = 'WasmParseError';
  }
}

export class EnvelopeDecodeError extends Error {
  constructor(message: string) {
    super(message);
    this.name = 'EnvelopeDecodeError';
  }
}

export class EnvelopeSizeError extends Error {
  constructor(message = `decompressed IDL exceeds maximum size of ${MAX_DECOMPRESSED_SIZE} bytes`) {
    super(message);
    this.name = 'EnvelopeSizeError';
  }
}

export class EnvelopeUtf8Error extends Error {
  constructor(message: string) {
    super(message);
    this.name = 'EnvelopeUtf8Error';
  }
}

type WasmInput = Uint8Array | ArrayBuffer | Blob;

const equalBytes = (left: Uint8Array, right: Uint8Array, offset: number): boolean => {
  if (left.length - offset < right.length) return false;
  for (let i = 0; i < right.length; i += 1) {
    if (left[offset + i] !== right[i]) return false;
  }
  return true;
};

const toOwnedU8a = async (input: WasmInput): Promise<Uint8Array> => {
  if (typeof Blob !== 'undefined' && input instanceof Blob) {
    return new Uint8Array(await input.arrayBuffer());
  }

  if (input instanceof Uint8Array) {
    return input;
  }

  if (input instanceof ArrayBuffer) {
    return new Uint8Array(input);
  }

  return new Uint8Array(await input.arrayBuffer());
};

const readUleb128 = (bytes: Uint8Array, offset: number): { value: number; offset: number } => {
  let result = 0;
  let shift = 0;

  for (let i = 0; i < 5; i += 1) {
    if (offset >= bytes.length) {
      throw new WasmParseError('truncated ULEB128');
    }

    const byte = bytes[offset];
    offset += 1;
    result += (byte & 0x7F) * 2 ** shift;

    if ((byte & 0x80) === 0) {
      if (result > 0xFFFFFFFF) throw new WasmParseError('ULEB128 overflow');
      return { value: result, offset };
    }

    shift += 7;
  }

  throw new WasmParseError('ULEB128 overflow');
};

const decodeUtf8 = (bytes: Uint8Array): string => new TextDecoder('utf-8', { fatal: true }).decode(bytes);

const decodeSectionName = (bytes: Uint8Array): { name: string; payload: Uint8Array } => {
  const nameLen = readUleb128(bytes, 0);
  const nameEnd = nameLen.offset + nameLen.value;
  if (nameEnd > bytes.length) {
    throw new WasmParseError('truncated custom section name');
  }

  const name = decodeUtf8(bytes.subarray(nameLen.offset, nameEnd));
  return { name, payload: bytes.subarray(nameEnd) };
};

const readStreamWithLimit = async (stream: ReadableStream<Uint8Array>): Promise<Uint8Array> => {
  const reader = stream.getReader();
  const chunks: Uint8Array[] = [];
  let total = 0;

  try {
    for (;;) {
      const { done, value } = await reader.read();
      if (done) break;
      if (!value) continue;

      total += value.byteLength;
      if (total > MAX_DECOMPRESSED_SIZE) {
        throw new EnvelopeSizeError();
      }
      chunks.push(value);
    }
  } finally {
    reader.releaseLock();
  }

  const out = new Uint8Array(total);
  let offset = 0;
  for (const chunk of chunks) {
    out.set(chunk, offset);
    offset += chunk.byteLength;
  }
  return out;
};

const inflateRaw = async (bytes: Uint8Array): Promise<Uint8Array> => {
  if (typeof DecompressionStream === 'undefined') {
    throw new EnvelopeDecodeError('DecompressionStream is not available');
  }

  try {
    const blobPart = bytes.buffer instanceof ArrayBuffer ? (bytes as Uint8Array<ArrayBuffer>) : bytes.slice();
    const stream = new Blob([blobPart]).stream().pipeThrough(new DecompressionStream('deflate-raw'));
    return await readStreamWithLimit(stream);
  } catch (e) {
    if (e instanceof EnvelopeSizeError) throw e;
    throw new EnvelopeDecodeError(String(e));
  }
};

const decodeEnvelope = async (data: Uint8Array): Promise<string | null> => {
  if (data.length < 2) return null;
  if (data[0] !== ENVELOPE_VERSION) return null;

  const flags = data[1];
  if ((flags & ~FLAG_COMPRESSED) !== 0) return null;

  const content = data.subarray(2);
  const idlBytes =
    (flags & FLAG_COMPRESSED) !== 0
      ? await inflateRaw(content)
      : (() => {
          if (content.length > MAX_DECOMPRESSED_SIZE) throw new EnvelopeSizeError();
          return content;
        })();

  try {
    return decodeUtf8(idlBytes);
  } catch (e) {
    throw new EnvelopeUtf8Error(String(e));
  }
};

export const extractIdlFromWasm = async (input: WasmInput): Promise<string | null> => {
  const wasm = await toOwnedU8a(input);

  if (!equalBytes(wasm, WASM_MAGIC, 0)) {
    throw new WasmParseError('invalid WASM magic');
  }
  if (!equalBytes(wasm, WASM_VERSION, 4)) {
    throw new WasmParseError('unsupported WASM version');
  }

  let offset = 8;
  while (offset < wasm.length) {
    const sectionId = wasm[offset];
    offset += 1;

    const sectionLen = readUleb128(wasm, offset);
    offset = sectionLen.offset;
    const sectionEnd = offset + sectionLen.value;
    if (sectionEnd > wasm.length) {
      throw new WasmParseError('truncated WASM section');
    }

    if (sectionId === SECTION_CUSTOM) {
      const { name, payload } = decodeSectionName(wasm.subarray(offset, sectionEnd));
      if (name === SECTION_NAME) {
        return decodeEnvelope(payload);
      }
    }

    offset = sectionEnd;
  }

  return null;
};

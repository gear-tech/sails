import { deflateRawSync } from 'node:zlib';
import { readFileSync } from 'node:fs';

import {
  EnvelopeDecodeError,
  EnvelopeSizeError,
  EnvelopeUtf8Error,
  WasmParseError,
  extractIdlFromWasm,
} from '..';

const enc = new TextEncoder();
const minimalWasm = () => new Uint8Array([0x00, 0x61, 0x73, 0x6D, 0x01, 0x00, 0x00, 0x00]);

const uleb = (value: number): number[] => {
  const out: number[] = [];
  do {
    let byte = value % 0x80;
    value = Math.floor(value / 0x80);
    if (value !== 0) byte |= 0x80;
    out.push(byte);
  } while (value !== 0);
  return out;
};

const customSection = (name: string, payload: Uint8Array): Uint8Array => {
  const nameBytes = enc.encode(name);
  const content = Uint8Array.from([...uleb(nameBytes.length), ...nameBytes, ...payload]);
  return Uint8Array.from([0, ...uleb(content.length), ...content]);
};

const wasmWithSections = (...sections: Uint8Array[]): Uint8Array => {
  const len = minimalWasm().length + sections.reduce((sum, section) => sum + section.length, 0);
  const out = new Uint8Array(len);
  let offset = 0;
  out.set(minimalWasm(), offset);
  offset += minimalWasm().length;
  for (const section of sections) {
    out.set(section, offset);
    offset += section.length;
  }
  return out;
};

const sailsSection = (payload: Uint8Array): Uint8Array => customSection('sails:idl', payload);
const envelope = (version: number, flags: number, content: Uint8Array): Uint8Array => {
  const out = new Uint8Array(content.length + 2);
  out[0] = version;
  out[1] = flags;
  out.set(content, 2);
  return out;
};

describe('extractIdlFromWasm', () => {
  test('throws on invalid wasm magic', async () => {
    await expect(extractIdlFromWasm(new Uint8Array([1, 2, 3]))).rejects.toBeInstanceOf(WasmParseError);
  });

  test('returns null when no sails:idl section exists', async () => {
    await expect(extractIdlFromWasm(minimalWasm())).resolves.toBeNull();
  });

  test('decodes raw and deflate envelopes', async () => {
    const raw = wasmWithSections(sailsSection(envelope(1, 0, enc.encode('service Raw {}'))));
    await expect(extractIdlFromWasm(raw)).resolves.toBe('service Raw {}');

    const compressed = deflateRawSync(enc.encode('service Zip {}'));
    const zipped = wasmWithSections(sailsSection(envelope(1, 1, compressed)));
    await expect(extractIdlFromWasm(zipped)).resolves.toBe('service Zip {}');
  });

  test('extracts shared Rust/JS snapshot fixture', async () => {
    const wasm = readFileSync('../test/fixtures/decoded/sails-idl-section.wasm');
    const expected = readFileSync('../test/fixtures/decoded/sails-idl-section.idl', 'utf8');

    await expect(extractIdlFromWasm(wasm)).resolves.toBe(expected);
  });

  test('returns null for forward-compatible envelope cases', async () => {
    await expect(extractIdlFromWasm(wasmWithSections(sailsSection(Uint8Array.from([2, 0, 1]))))).resolves.toBeNull();
    await expect(extractIdlFromWasm(wasmWithSections(sailsSection(Uint8Array.from([1, 2, 1]))))).resolves.toBeNull();
    await expect(extractIdlFromWasm(wasmWithSections(sailsSection(Uint8Array.from([1]))))).resolves.toBeNull();
  });

  test('uses first matching section', async () => {
    const wasm = wasmWithSections(
      sailsSection(envelope(1, 0, enc.encode('first'))),
      sailsSection(envelope(1, 0, enc.encode('second'))),
    );
    await expect(extractIdlFromWasm(wasm)).resolves.toBe('first');
  });

  test('throws typed envelope errors', async () => {
    await expect(
      extractIdlFromWasm(wasmWithSections(sailsSection(Uint8Array.from([1, 1, 0xFF, 0xFE, 0xFD])))),
    ).rejects.toBeInstanceOf(EnvelopeDecodeError);

    await expect(
      extractIdlFromWasm(wasmWithSections(sailsSection(envelope(1, 0, Uint8Array.from([0xFF, 0xFE]))))),
    ).rejects.toBeInstanceOf(EnvelopeUtf8Error);

    const tooLarge = new Uint8Array(1024 * 1024 + 1);
    await expect(
      extractIdlFromWasm(wasmWithSections(sailsSection(envelope(1, 0, tooLarge)))),
    ).rejects.toBeInstanceOf(EnvelopeSizeError);
  });

  test('handles SharedArrayBuffer-backed input', async () => {
    const wasm = wasmWithSections(sailsSection(envelope(1, 0, enc.encode('shared'))));
    const shared = new Uint8Array(new SharedArrayBuffer(wasm.length));
    shared.set(wasm);

    await expect(extractIdlFromWasm(shared)).resolves.toBe('shared');
  });

  test('handles unsigned u32 ULEB128 section lengths', async () => {
    const wasm = Uint8Array.from([...minimalWasm(), 0, ...uleb(0x8000_0000)]);

    await expect(extractIdlFromWasm(wasm)).rejects.toThrow('truncated WASM section');
  });

  test('throws on oversized ULEB128 section lengths', async () => {
    const wasm = Uint8Array.from([...minimalWasm(), 0, ...uleb(0x1_0000_0000)]);

    await expect(extractIdlFromWasm(wasm)).rejects.toThrow('ULEB128 overflow');
  });
});

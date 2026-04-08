import { describe, test, expect } from 'bun:test';
import { SailsMessageHeader, InterfaceId } from 'sails-js-parser-idl-v2';

describe('Header operations', () => {
  test('build and parse header round-trip', () => {
    const iid = InterfaceId.fromString('0x579d6daba41b7d82');
    const header = SailsMessageHeader.v1(iid, 0, 1);
    const bytes = header.toBytes();

    expect(bytes.length).toBe(16);
    // Magic: GM
    expect(bytes[0]).toBe(0x47);
    expect(bytes[1]).toBe(0x4d);
    // Version: 1
    expect(bytes[2]).toBe(1);
    // Header length: 16
    expect(bytes[3]).toBe(16);

    // Parse it back
    const { header: parsed, offset } = SailsMessageHeader.tryReadBytes(bytes, 0);
    expect(offset).toBe(16);
    expect(parsed.version).toBe(1);
    expect(parsed.hlen).toBe(16);
    expect(parsed.interfaceId.toString()).toBe('0x579d6daba41b7d82');
    expect(parsed.entryId).toBe(0);
    expect(parsed.routeIdx).toBe(1);
  });

  test('InterfaceId conversions', () => {
    const fromStr = InterfaceId.fromString('0x579d6daba41b7d82');
    const fromU64 = InterfaceId.fromU64(fromStr.asU64());

    expect(fromStr.toString()).toBe('0x579d6daba41b7d82');
    expect(fromU64.toString()).toBe('0x579d6daba41b7d82');
    expect(fromStr.asU64()).toBe(fromU64.asU64());
  });

  test('detect Sails magic bytes', () => {
    const iid = InterfaceId.fromString('0x579d6daba41b7d82');
    const header = SailsMessageHeader.v1(iid, 5, 0);
    const bytes = header.toBytes();

    const { ok, header: parsed } = SailsMessageHeader.tryFromBytes(bytes);
    expect(ok).toBe(true);
    expect(parsed).toBeDefined();
    expect(parsed!.entryId).toBe(5);
  });

  test('tryFromBytes fails for non-Sails bytes', () => {
    const bytes = new Uint8Array([0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e, 0x0f]);
    const { ok } = SailsMessageHeader.tryFromBytes(bytes);
    expect(ok).toBe(false);
  });
});

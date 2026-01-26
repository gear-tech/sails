import type { InterfaceIdInput, IInterfaceId } from "./idl-v2-types";

const HEX_CHUNK_RE = /^[0-9a-fA-F]{2}$/;

export class InterfaceId implements IInterfaceId {
  public readonly bytes: Uint8Array;

  private constructor(bytes: Uint8Array) {
    if (bytes.length !== 8) {
      throw new RangeError(`expected 8 bytes, got ${bytes.length}`);
    }
    this.bytes = bytes;
  }

  public static zero(): InterfaceId {
    return new InterfaceId(new Uint8Array(8));
  }

  public static fromBytes32(bytes: ArrayLike<number>): InterfaceId {
    if (bytes.length < 32) {
      throw new RangeError(`expected 32 bytes, got ${bytes.length}`);
    }

    return InterfaceId.fromBytes8(Array.from(bytes).slice(0, 8));
  }

  public static fromBytes8(bytes: ArrayLike<number>): InterfaceId {
    if (bytes.length < 8) {
      throw new RangeError(`expected 8 bytes, got ${bytes.length}`);
    }

    const out = new Uint8Array(8);
    for (let i = 0; i < 8; i += 1) {
      out[i] = bytes[i] ?? 0;
    }
    return new InterfaceId(out);
  }

  public static fromU64(value: bigint | number): InterfaceId {
    const big = typeof value === "number" ? BigInt(value) : value;
    if (big < 0n || big > 0xffff_ffff_ffff_ffffn) {
      throw new RangeError("u64 value out of range");
    }

    const out = new Uint8Array(8);
    let temp = big;
    for (let i = 7; i >= 0; i -= 1) {
      out[i] = Number(temp & 0xffn);
      temp >>= 8n;
    }
    return new InterfaceId(out);
  }

  public static fromString(value: string): InterfaceId {
    let hex = value.trim();
    if (hex.startsWith("0x") || hex.startsWith("0X")) {
      hex = hex.slice(2);
    }

    if (hex.length !== 16) {
      throw new RangeError(`expected 16 hex digits (8 bytes), got ${hex.length}`);
    }

    const out = new Uint8Array(8);
    for (let i = 0; i < 8; i += 1) {
      const chunk = hex.slice(i * 2, i * 2 + 2);
      if (!HEX_CHUNK_RE.test(chunk)) {
        throw new RangeError(`invalid hex byte: ${chunk}`);
      }
      out[i] = Number.parseInt(chunk, 16);
    }

    return new InterfaceId(out);
  }

  public static tryReadBytes(bytes: Uint8Array, offset = 0): { id: InterfaceId; offset: number } {
    if (bytes.length - offset < 8) {
      throw new RangeError("Insufficient bytes for interface ID");
    }

    const slice = bytes.slice(offset, offset + 8);
    return { id: new InterfaceId(slice), offset: offset + 8 };
  }

  public static tryFromBytes(bytes: Uint8Array): InterfaceId {
    return InterfaceId.tryReadBytes(bytes, 0).id;
  }

  public static from(input: InterfaceIdInput): InterfaceId {
    if (input instanceof InterfaceId) {
      return input;
    }
    if (typeof input === "string") {
      return InterfaceId.fromString(input);
    }
    if (typeof input === "number" || typeof input === "bigint") {
      return InterfaceId.fromU64(input);
    }
    if (input instanceof Uint8Array) {
      return InterfaceId.fromBytes8(input);
    }
    if (InterfaceId.isInterfaceIdLike(input)) {
      return InterfaceId.fromBytes8(input.bytes);
    }
    return InterfaceId.fromBytes8(input);
  }

  public asBytes(): Uint8Array {
    return this.bytes.slice();
  }

  public from(input: InterfaceIdInput): InterfaceId {
    return InterfaceId.from(input);
  }

  public asU64(): bigint {
    let out = 0n;
    for (const byte of this.bytes) {
      out = (out << 8n) | BigInt(byte);
    }
    return out;
  }

  public toString(): string {
    let out = "0x";
    for (const byte of this.bytes) {
      out += byte.toString(16).padStart(2, "0");
    }
    return out;
  }

  public toJSON(): string {
    return this.toString();
  }

  private static isInterfaceIdLike(value: InterfaceIdInput): value is IInterfaceId {
    return (
      typeof value === "object" &&
      value !== null &&
      "bytes" in value &&
      value.bytes instanceof Uint8Array
    );
  }
}

import { InterfaceId } from "./interface_id";

export const HIGHEST_SUPPORTED_VERSION = 1;
export const MAGIC_BYTES = new Uint8Array([0x47, 0x4d]);
export const MINIMAL_HLEN = 16;

const equalBytes = (a: Uint8Array, b: Uint8Array): boolean => {
  if (a.length !== b.length) {
    return false;
  }
  for (let i = 0; i < a.length; i += 1) {
    if (a[i] !== b[i]) {
      return false;
    }
  }
  return true;
};

const ensureVersion = (version: number): number => {
  if (version === 0 || version > HIGHEST_SUPPORTED_VERSION) {
    throw new RangeError("Unsupported Sails version");
  }
  return version;
};

const ensureHeaderLength = (hlen: number): number => {
  if (hlen < MINIMAL_HLEN) {
    throw new RangeError("Header length is less than minimal Sails header length");
  }
  return hlen;
};

const tryReadMagic = (bytes: Uint8Array, offset: number): number => {
  if (bytes.length - offset < MAGIC_BYTES.length) {
    throw new RangeError("Insufficient bytes for magic");
  }
  const slice = bytes.slice(offset, offset + 2);
  if (!equalBytes(slice, MAGIC_BYTES)) {
    throw new RangeError("Invalid Sails magic bytes");
  }
  return offset + 2;
};

export class SailsMessageHeader {
  public readonly version: number;
  public readonly hlen: number;
  public readonly interface_id: InterfaceId;
  public readonly route_idx: number;
  public readonly entry_id: number;

  private constructor(
    version: number,
    hlen: number,
    interface_id: InterfaceId,
    route_idx: number,
    entry_id: number
  ) {
    this.version = ensureVersion(version);
    this.hlen = ensureHeaderLength(hlen);
    this.interface_id = interface_id;
    this.route_idx = route_idx;
    this.entry_id = entry_id;
  }

  public static new(
    version: number,
    hlen: number,
    interface_id: InterfaceId,
    route_idx: number,
    entry_id: number
  ): SailsMessageHeader {
    return new SailsMessageHeader(version, hlen, interface_id, route_idx, entry_id);
  }

  public static v1(interface_id: InterfaceId, entry_id: number, route_idx: number): SailsMessageHeader {
    return new SailsMessageHeader(1, MINIMAL_HLEN, interface_id, route_idx, entry_id);
  }

  public toBytes(): Uint8Array {
    const bytes = new Uint8Array(this.hlen);
    let offset = 0;

    bytes.set(MAGIC_BYTES, offset);
    offset += 2;
    bytes[offset] = this.version;
    offset += 1;
    bytes[offset] = this.hlen;
    offset += 1;
    bytes.set(this.interface_id.asBytes(), offset);
    offset += 8;

    const entry = this.entry_id & 0xffff;
    bytes[offset] = entry & 0xff;
    bytes[offset + 1] = (entry >> 8) & 0xff;
    offset += 2;

    bytes[offset] = this.route_idx & 0xff;
    bytes[offset + 1] = 0;

    return bytes;
  }

  public static tryReadBytes(
    bytes: Uint8Array,
    offset = 0
  ): { header: SailsMessageHeader; offset: number } {
    if (bytes.length - offset < MINIMAL_HLEN) {
      throw new RangeError("Insufficient bytes for header");
    }

    offset = tryReadMagic(bytes, offset);

    if (bytes.length - offset < 1) {
      throw new RangeError("Insufficient bytes for version");
    }
    const version = ensureVersion(bytes[offset]);
    offset += 1;

    if (bytes.length - offset < 1) {
      throw new RangeError("Insufficient bytes for header length");
    }
    const hlen = ensureHeaderLength(bytes[offset]);
    offset += 1;

    const interfaceResult = InterfaceId.tryReadBytes(bytes, offset);
    const interfaceId = interfaceResult.id;
    offset = interfaceResult.offset;

    const entryId = bytes[offset] | (bytes[offset + 1] << 8);
    const routeId = bytes[offset + 2];
    const reserved = bytes[offset + 3];

    if (version === 1 && reserved !== 0) {
      throw new RangeError("Reserved byte must be zero in version 1");
    }

    offset += 4;

    return {
      header: new SailsMessageHeader(version, hlen, interfaceId, routeId, entryId),
      offset,
    };
  }

  public static tryFromBytes(bytes: Uint8Array): SailsMessageHeader {
    return SailsMessageHeader.tryReadBytes(bytes, 0).header;
  }

  public tryMatchInterfaces(interfaces: Array<[InterfaceId, number]>): MatchedInterface {
    let sameInterfaceIds = 0;
    let hasRoute = false;

    for (const [id, programRouteId] of interfaces) {
      if (equalBytes(id.asBytes(), this.interface_id.asBytes())) {
        sameInterfaceIds += 1;
        if (!hasRoute) {
          hasRoute = this.route_idx === programRouteId;
        }
      }
    }

    if (sameInterfaceIds === 0) {
      throw new RangeError("No matching interface ID found");
    }
    if (this.route_idx === 0 && sameInterfaceIds > 1) {
      throw new RangeError("Can't infer the interface by route id 0, many instances");
    }
    if (!hasRoute && this.route_idx !== 0) {
      throw new RangeError("No matching route ID found for the interface ID");
    }

    return new MatchedInterface(this.interface_id, this.route_idx, this.entry_id);
  }
}

export class MatchedInterface {
  public readonly interfaceId: InterfaceId;
  public readonly routeId: number;
  public readonly entryId: number;

  public constructor(interfaceId: InterfaceId, routeId: number, entryId: number) {
    this.interfaceId = interfaceId;
    this.routeId = routeId;
    this.entryId = entryId;
  }

  public intoInner(): [InterfaceId, number, number] {
    return [this.interfaceId, this.routeId, this.entryId];
  }
}

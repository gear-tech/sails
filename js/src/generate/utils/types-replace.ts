const KNOWN_TYPES = new Map<string, string>([
  ['bool', 'boolean'],
  ['u8', 'number'],
  ['u16', 'number'],
  ['u32', 'bigint'],
  ['u64', 'bigint'],
  ['u128', 'bigint'],
  ['u256', 'bigint'],
  ['i8', 'number'],
  ['i16', 'number'],
  ['i32', 'bigint'],
  ['i64', 'bigint'],
  ['i128', 'bigint'],
  ['i256', 'bigint'],
  ['String', 'string'],
  ['str', 'string'],
  ['Vec<u8>', 'Uint8Array | `0x${string}`'],
  ['[u8;64]', 'Uint8Array | `0x${string}`'],
  ['[u8;32]', 'Uint8Array | `0x${string}`'],
  ['Bytes', 'Uint8Array | `0x${string}`'],
  ['ActorId', '`0x${string}`'],
  ['Null', 'null'],
  ['null', 'null'],
]);

export function getTSType(name: string) {
  if (KNOWN_TYPES.has(name)) {
    return KNOWN_TYPES.get(name);
  }
  return name;
}

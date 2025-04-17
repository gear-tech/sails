export function isHex(value: string): value is `0x${string}` {
  return /^0x[0-9a-fA-F]+$/.test(value);
}

export function toHex(value: Uint8Array): `0x${string}` {
  return `0x${[...value].map((byte) => byte.toString(16).padStart(2, '0')).join('')}`;
}

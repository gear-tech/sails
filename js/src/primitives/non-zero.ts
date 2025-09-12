type UintType = string | number | bigint;

const UNonZeroBase = <T>(value: UintType, size: 8 | 16 | 32 | 64 | 128 | 256): T => {
  const _value = BigInt(value);

  if (_value <= 0n) {
    throw new Error('Value is not non-zero');
  }

  if (_value >= 2n ** BigInt(size)) {
    throw new Error('Value is too large');
  }

  return (size <= 32 ? Number(value) : value) as T;
};

export const NonZeroU8 = (value: UintType): NonZeroU8 => UNonZeroBase(value, 8);
export const NonZeroU16 = (value: UintType): NonZeroU16 => UNonZeroBase(value, 16);
export const NonZeroU32 = (value: UintType): NonZeroU32 => UNonZeroBase(value, 32);
export const NonZeroU64 = (value: UintType): NonZeroU64 => UNonZeroBase(value, 64);
export const NonZeroU128 = (value: UintType): NonZeroU128 => UNonZeroBase(value, 128);
export const NonZeroU256 = (value: UintType): NonZeroU256 => UNonZeroBase(value, 256);

export type NonZeroU8 = number;
export type NonZeroU16 = number;
export type NonZeroU32 = number;
export type NonZeroU64 = UintType;
export type NonZeroU128 = UintType;
export type NonZeroU256 = UintType;

import { HexString } from '@gear-js/api';
import { isHex, toHex } from 'sails-js-util';

const FixedSizedHash = (value: HexString | Uint8Array, size: number): HexString => {
  if (typeof value === 'string') {
    if (!isHex(value)) {
      throw new Error('Value is not a hex string');
    }

    if (value.length !== size * 2 + 2) {
      throw new Error('Value has incorrect length');
    }
    return value.toLowerCase() as HexString;
  } else if (value.length !== size) {
    throw new Error('Value has incorrect length');
  }

  return toHex(value);
};

export const H160 = (value: HexString | Uint8Array) => FixedSizedHash(value, 20);
export const H256 = (value: HexString | Uint8Array) => FixedSizedHash(value, 32);
export const ActorId = (value: HexString | Uint8Array) => FixedSizedHash(value, 32);
export const CodeId = (value: HexString | Uint8Array) => FixedSizedHash(value, 32);
export const MessageId = (value: HexString | Uint8Array) => FixedSizedHash(value, 32);

export type H160 = HexString;
export type H256 = HexString;
export type ActorId = HexString;
export type CodeId = HexString;
export type MessageId = HexString;

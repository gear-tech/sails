import { HexString } from '@gear-js/api';
import { isHex } from '@polkadot/util';

const FixedSizedHash = (value: HexString | Uint8Array, size: number) => {
  if (typeof value === 'string') {
    if (!isHex(value)) {
      throw new Error('Value is not a hex string');
    }

    if (value.length !== size * 2 + 2) {
      throw new Error('Value has incorrect length');
    }
  } else if (value.length !== size) {
    throw new Error('Value has incorrect length');
  }

  return value;
};

export const H160 = (value: HexString | Uint8Array) => FixedSizedHash(value, 20);
export const H256 = (value: HexString | Uint8Array) => FixedSizedHash(value, 32);
export const ActorId = (value: HexString | Uint8Array) => FixedSizedHash(value, 32);
export const CodeId = (value: HexString | Uint8Array) => FixedSizedHash(value, 32);
export const MessageId = (value: HexString | Uint8Array) => FixedSizedHash(value, 32);

export type H160 = HexString | Uint8Array;
export type H256 = HexString | Uint8Array;
export type ActorId = HexString | Uint8Array;
export type CodeId = HexString | Uint8Array;
export type MessageId = HexString | Uint8Array;

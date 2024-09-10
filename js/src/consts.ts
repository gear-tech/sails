import { u8aToHex } from '@polkadot/util';

export const ZERO_ADDRESS = u8aToHex(new Uint8Array(32));

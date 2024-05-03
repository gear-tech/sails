import { u8aToString, hexToU8a, compactFromU8aLim } from '@polkadot/util';
import { HexString } from '@gear-js/api';

/**
 * ## Get service name prefix
 * @param payload in hex string format
 * @returns Name of the service
 */
export const getServiceNamePrefix = (payload: HexString): string => {
  const _payload = hexToU8a(payload);
  const [offset, limit] = compactFromU8aLim(_payload);

  return u8aToString(_payload.subarray(offset, limit + offset));
};

/**
 * ## Get function (or event) name prefix
 * @param payload in hex string format
 * @returns Name of the function
 */
export function getFnNamePrefix(payload: HexString) {
  const _payload = hexToU8a(payload);

  const [sOff, sLim] = compactFromU8aLim(_payload);
  const serviceOffset = sOff + sLim;

  const [offset, limit] = compactFromU8aLim(_payload.subarray(serviceOffset));

  return u8aToString(_payload.subarray(serviceOffset + offset, serviceOffset + offset + limit));
}

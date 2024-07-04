import { u8aToString, hexToU8a, compactFromU8aLim } from '@polkadot/util';
import { HexString } from '@gear-js/api';

/**
 * ## Get service name prefix
 * @param payload in hex string format
 * @returns Name of the service
 */
export function getServiceNamePrefix(payload: HexString): string;

/**
 * ## Get service name prefix and bytes length
 * @param payload in hex string format
 * @param withBytesLength flag
 * @returns Name of the service and bytes length
 */
export function getServiceNamePrefix(
  payload: HexString,
  withBytesLength: true,
): { service: string; bytesLength: number };

export function getServiceNamePrefix(
  payload: HexString,
  withBytesLength: boolean = false,
): string | { service: string; bytesLength: number } {
  const _payload = hexToU8a(payload);
  const [offset, limit] = compactFromU8aLim(_payload);

  const prefix = u8aToString(_payload.subarray(offset, limit + offset));

  return withBytesLength ? { service: prefix, bytesLength: limit + offset } : prefix;
}

/**
 * ## Get function (or event) name prefix
 * @param payload in hex string format
 * @returns Name of the function
 */
export function getFnNamePrefix(payload: HexString): string;

/**
 * ## Get function (or event) name prefix and bytes length
 * @param payload in hex string format
 * @param withBytesLength flag
 * @returns Name of the function and bytes length
 */
export function getFnNamePrefix(payload: HexString, withBytesLength: true): { fn: string; bytesLength: number };

export function getFnNamePrefix(
  payload: HexString,
  withBytesLength: boolean = false,
): string | { fn: string; bytesLength: number } {
  const _payload = hexToU8a(payload);

  const [sOff, sLim] = compactFromU8aLim(_payload);
  const serviceOffset = sOff + sLim;

  const [offset, limit] = compactFromU8aLim(_payload.subarray(serviceOffset));

  const prefix = u8aToString(_payload.subarray(serviceOffset + offset, serviceOffset + offset + limit));

  return withBytesLength ? { fn: prefix, bytesLength: offset + limit } : prefix;
}

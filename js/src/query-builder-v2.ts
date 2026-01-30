import { decodeAddress, GearApi, HexString } from '@gear-js/api';
import { isHex, u8aConcat, u8aToHex } from '@polkadot/util';
import { TypeRegistry } from '@polkadot/types';
import { getPayloadMethod } from 'sails-js-util';
import type { SailsMessageHeader } from 'sails-js-parser-v2';

import { throwOnErrorReply } from './utils.js';
import { ZERO_ADDRESS } from './consts.js';

export class QueryBuilder<ResultType = unknown> {
  private _prefixByteLength: number;
  private _payload: Uint8Array;
  private _value: bigint = 0n;
  private _gasLimit?: bigint;
  private _originAddress?: HexString;
  private _atBlock?: HexString;

  constructor(
    private _api: GearApi,
    private _registry: TypeRegistry,
    private _programId: HexString,
    header: SailsMessageHeader,
    payload: unknown | null,
    payloadType: string | null,
    private _responseType: string,
  ) {
    const encodedHeader = header.toBytes();
    const data = payloadType === null ? new Uint8Array() : this._registry.createType<any>(payloadType, payload).toU8a();

    this._payload = u8aConcat(encodedHeader, data);
    this._prefixByteLength = encodedHeader.length;
  }

  /**
   * Get the payload of the query as a hexadecimal string.
   */
  public get payload(): HexString {
    return u8aToHex(this._payload);
  }

  /**
   * Set the value of the query (default: 0).
   */
  public withValue(value: bigint): this {
    this._value = value;

    return this;
  }

  /**
   * Set the origin address of the query (default: Zero Address).
   * @param address
   */
  public withAddress(address: string): this {
    if (isHex(address)) {
      this._originAddress = address;
    } else {
      try {
        this._originAddress = decodeAddress(address);
      } catch {
        throw new Error('Invalid address.');
      }
    }

    return this;
  }

  /**
   * Set the gas limit of the query (default: max).
   * @param value
   */
  public withGasLimit(value: bigint): this {
    this._gasLimit = value;
    return this;
  }

  /**
   * Set the block hash to query at (default: latest block).
   * @param hash
   */
  public atBlock(hash: HexString): this {
    this._atBlock = hash;
    return this;
  }

  /**
   * Execute the query and return the result.
   */
  public async call(): Promise<ResultType> {
    const result = await this._api.message.calculateReply({
      destination: this._programId,
      origin: this._originAddress || ZERO_ADDRESS,
      payload: this._payload,
      value: this._value,
      gasLimit: this._gasLimit || this._api.blockGasLimit.toBigInt(),
      at: this._atBlock,
    });

    throwOnErrorReply(result.code, result.payload.toU8a(), this._api.specVersion, this._registry);

    const responseWOPrefix = result.payload.slice(this._prefixByteLength);

    const responseDecoded = this._registry.createType<any>(this._responseType, responseWOPrefix);

    return responseDecoded[getPayloadMethod(this._responseType)]() as ResultType;
  }
}

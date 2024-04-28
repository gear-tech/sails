import { GearApi, HexString, MessageQueuedData, decodeAddress } from '@gear-js/api';
import { SignerOptions, SubmittableExtrinsic } from '@polkadot/api/types';
import { IKeyringPair, ISubmittableResult } from '@polkadot/types/types';
import { TypeRegistry, u128, u64 } from '@polkadot/types';

export interface IMethodReturnType<T> {
  msgId: string;
  blockHash: string;
  response: () => Promise<T>;
}

export class TransactionBuilder<ResponseType> {
  private _account: string | IKeyringPair;
  private _signerOptions: Partial<SignerOptions>;
  private _tx: SubmittableExtrinsic<'promise', ISubmittableResult>;
  public programId: HexString;

  constructor(
    api: GearApi,
    registry: TypeRegistry,
    extrinsic: 'send_message',
    payload: unknown,
    payloadType: string,
    responseType: string,
    programId: HexString,
  );
  constructor(
    api: GearApi,
    registry: TypeRegistry,
    extrinsic: 'upload_program',
    payload: unknown,
    payloadType: string,
    responseType: string,
    code: Uint8Array | ArrayBufferLike | HexString,
  );
  constructor(
    api: GearApi,
    registry: TypeRegistry,
    extrinsic: 'create_program',
    payload: unknown,
    payloadType: string,
    responseType: string,
    codeId: HexString,
  );
  constructor(
    private _api: GearApi,
    private _registry: TypeRegistry,
    extrinsic: 'send_message' | 'upload_program' | 'create_program',
    payload: unknown,
    payloadType: string,
    private _responseType: string,
    _programIdOrCodeOrCodeId: HexString | Uint8Array | ArrayBufferLike,
  ) {
    const _payload = this._registry.createType<any>(payloadType, payload);
    switch (extrinsic) {
      case 'send_message': {
        this.programId = _programIdOrCodeOrCodeId as HexString;
        this._tx = this._api.message.send({
          destination: this.programId,
          gasLimit: 0,
          payload: _payload.toU8a(),
          value: 0,
        });
        break;
      }
      case 'upload_program': {
        const { programId, extrinsic } = this._api.program.upload({
          code: _programIdOrCodeOrCodeId as Uint8Array,
          gasLimit: 0,
          initPayload: _payload.toU8a(),
        });
        this.programId = programId;
        this._tx = extrinsic;
        break;
      }
      case 'create_program': {
        const { programId, extrinsic } = this._api.program.create({
          codeId: _programIdOrCodeOrCodeId as HexString,
          gasLimit: 0,
          initPayload: _payload.toU8a(),
        });
        this.programId = programId;
        this._tx = extrinsic;
        break;
      }
    }
  }

  private _getGas(gas: u64, increaseGas: number): u64 {
    if (increaseGas === 0) return gas;
    if (increaseGas < 0 || increaseGas > 100) throw new Error('Invalid increaseGas value (0-100)');

    return this._registry.createType('u64', gas.add(gas.muln(increaseGas / 100)));
  }

  private _getValue(value: bigint): u128 {
    return this._registry.createType('u128', value);
  }

  private _setTxArg(index: number, value: unknown) {
    switch (this._tx.method.method) {
      case 'uploadProgram': {
        this._tx = this._api.tx.gear.uploadProgram(...this._tx.args.map((arg, i) => (i === index ? value : arg)));
        break;
      }
      case 'createProgram': {
        this._tx = this._api.tx.gear.createProgram(...this._tx.args.map((arg, i) => (i === index ? value : arg)));
        break;
      }
      case 'sendMessage': {
        this._tx = this._api.tx.gear.sendMessage(...this._tx.args.map((arg, i) => (i === index ? value : arg)));
        break;
      }
    }
  }

  /**
   * ## Calculate gas for transaction
   * @param allowOtherPanics Allow panics in other contracts to be triggered (default: false)
   * @param increaseGas Increase the gas limit by a percentage from 0 to 100 (default: 0)
   * @returns
   */
  public async calculateGas(allowOtherPanics = false, increaseGas = 0) {
    if (!this._account) throw new Error('Account is required. Use withAccount() method to set account.');

    const source = decodeAddress(typeof this._account === 'string' ? this._account : this._account.address);

    switch (this._tx.method.method) {
      case 'uploadProgram': {
        const gas = await this._api.program.calculateGas.initUpload(
          source,
          this._tx.args[0].toHex(),
          this._tx.args[2].toHex(),
          this._tx.args[4] as u128,
          allowOtherPanics,
        );

        this._setTxArg(3, this._getGas(gas.min_limit, increaseGas));

        break;
      }
      case 'createProgram': {
        const gas = await this._api.program.calculateGas.initCreate(
          source,
          this._tx.args[0].toHex(),
          this._tx.args[2].toHex(),
          this._tx.args[4] as u128,
          allowOtherPanics,
        );

        this._setTxArg(3, this._getGas(gas.min_limit, increaseGas));

        break;
      }
      case 'sendMessage': {
        const gas = await this._api.program.calculateGas.handle(
          source,
          this._tx.args[0].toHex(),
          this._tx.args[1].toHex(),
          this._tx.args[3] as u128,
          allowOtherPanics,
        );

        this._setTxArg(2, this._getGas(gas.min_limit, increaseGas));

        break;
      }
      default: {
        throw new Error('Unknown extrinsic');
      }
    }

    return this;
  }

  /**
   * ## Set account for transaction
   * @param account
   * @param signerOptions
   */
  public withAccount(account: string | IKeyringPair, signerOptions?: Partial<SignerOptions>) {
    this._account = account;
    if (signerOptions) {
      this._signerOptions = signerOptions;
    }
    return this;
  }

  /**
   * ## Set value for transaction
   * @param value
   */
  public async withValue(value: bigint) {
    switch (this._tx.method.method) {
      case 'uploadProgram':
      case 'createProgram': {
        this._setTxArg(4, this._getValue(value));
        break;
      }
      case 'sendMessage': {
        this._setTxArg(3, this._getValue(value));
        break;
      }
      default: {
        throw new Error('Unknown extrinsic');
      }
    }
    return this;
  }

  /**
   * ## Set gas for transaction
   * @param gas
   */
  public async withGas(gas: bigint) {
    switch (this._tx.method.method) {
      case 'uploadProgram':
      case 'createProgram': {
        this._setTxArg(3, this._registry.createType('u64', gas));
        break;
      }
      case 'sendMessage': {
        this._setTxArg(2, this._registry.createType('u64', gas));
        break;
      }
      default: {
        throw new Error('Unknown extrinsic');
      }
    }

    return this;
  }

  /**
   * ## Sign and send transaction
   */
  public async signAndSend(): Promise<IMethodReturnType<ResponseType>> {
    const { msgId, blockHash } = await new Promise<{ msgId: HexString; blockHash: HexString }>((resolve, reject) =>
      this._tx
        .signAndSend(this._account, this._signerOptions, ({ events, status }) => {
          if (status.isInBlock) {
            let msgId: HexString;

            events.forEach(({ event }) => {
              const { method, section, data } = event;
              if (method === 'MessageQueued' && section === 'gear') {
                msgId = (data as MessageQueuedData).id.toHex();
              } else if (method === 'ExtrinsicSuccess') {
                resolve({ msgId, blockHash: status.asInBlock.toHex() });
              } else if (method === 'ExtrinsicFailed') {
                reject(this._api.getExtrinsicFailedError(event));
              }
            });
          }
        })
        .catch((error) => {
          reject(error.message);
        }),
    );

    return {
      msgId,
      blockHash,
      response: async () => {
        const {
          data: { message },
        } = await this._api.message.getReplyEvent(this.programId, msgId, blockHash);

        if (!message.details.unwrap().code.isSuccess) {
          throw new Error(this._registry.createType('String', message.payload).toString());
        }

        return this._registry
          .createType<any>(`(String, String, ${this._responseType})`, message.payload)[2]
          .toJSON() as ResponseType;
      },
    };
  }
}

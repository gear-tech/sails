import { AddressOrPair, SignerOptions, SubmittableExtrinsic } from '@polkadot/api/types';
import { IKeyringPair, ISubmittableResult } from '@polkadot/types/types';
import { ProgramBase } from './program-base';
import { GasInfo, decodeAddress } from '@gear-js/api';
import { u128, u64 } from '@polkadot/types';

export class TransactionBuilder {
  private _account: string | IKeyringPair;
  private _signerOpotions?: Partial<SignerOptions>;
  private _value: bigint;
  private _gas: u64;

  constructor(private _program: ProgramBase, private _tx: SubmittableExtrinsic<'promise', ISubmittableResult>) {}

  private _getGas(gas: u64, increaseGas: number): u64 {
    if (increaseGas === 0) return gas;
    if (increaseGas < 0 || increaseGas > 100) throw new Error('Invalid increaseGas value (0-100)');

    return this._program.registry.createType('u64', gas.add(gas.muln(increaseGas / 100)));
  }

  private _getValue(value: bigint): u128 {
    return this._program.registry.createType('u128', value);
  }

  /**
   * ## Calculate gas for transaction
   * @param allowOtherPanics Allow panics in other contracts to be triggered (default: false)
   * @param increaseGas Increase the gas limit by a percentage from 0 to 100 (default: 0)
   * @returns
   */
  public async calculateGas(allowOtherPanics = false, increaseGas = 0): Promise<TransactionBuilder> {
    if (!this._account) throw new Error('Account is required. Use withAccount() method to set account.');

    const source = decodeAddress(typeof this._account === 'string' ? this._account : this._account.address);

    switch (this._tx.method.method) {
      case 'upload_program': {
        const gas = await this._program.api.program.calculateGas.initUpload(
          source,
          this._tx.args[0].toU8a(),
          this._tx.args[1].toU8a(),
          this._tx.args[2] as u128,
          allowOtherPanics,
        );
        this._tx.args[3] = this._getGas(gas.min_limit, increaseGas);
        break;
      }
      case 'create_program': {
        const gas = await this._program.api.program.calculateGas.initCreate(
          source,
          this._tx.args[0].toHex(),
          this._tx.args[1].toU8a(),
          this._tx.args[2] as u128,
          allowOtherPanics,
        );
        this._tx.args[3] = this._getGas(gas.min_limit, increaseGas);
        break;
      }
      case 'send_message': {
        const gas = await this._program.api.program.calculateGas.handle(
          source,
          this._tx.args[0].toHex(),
          this._tx.args[1].toU8a(),
          this._tx.args[3] as u128,
          allowOtherPanics,
        );
        this._tx.args[2] = this._getGas(gas.min_limit, increaseGas);
        break;
      }
      default: {
        throw new Error('Unknown extrinsic');
      }
    }

    return this;
  }

  /**
   *
   * @param account
   * @param signerOptions
   * @returns
   */
  public withAccount(account: string | IKeyringPair, signerOptions?: Partial<SignerOptions>) {
    this._account = account;
    this._signerOpotions = signerOptions;
    return this;
  }

  public async withValue(value: bigint) {
    switch (this._tx.method.method) {
      case 'upload_program':
      case 'create_program': {
        this._tx.args[4] = this._getValue(value);
        break;
      }
      case 'send_message': {
        this._tx.args[3] = this._getValue(value);
        break;
      }
      default: {
        throw new Error('Unknown extrinsic');
      }
    }
    return this;
  }

  public signAndSend() {
    const signedTx = this._tx.signAndSend(this._account, this._signerOpotions);
  }
}

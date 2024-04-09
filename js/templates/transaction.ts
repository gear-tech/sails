import { GearApi, HexString, MessageQueuedData, UserMessageSentData, decodeAddress } from '@gear-js/api';
import { SignerOptions, SubmittableExtrinsic } from '@polkadot/api/types';
import { TypeRegistry } from '@polkadot/types';
import { IKeyringPair, ISubmittableResult } from '@polkadot/types/types';
import { Registry } from '@polkadot/types-codec/types/registry';
import { ReplaySubject } from 'rxjs';

export interface IMethodReturnType<T> {
  msgId: string;
  blockHash: string;
  response: () => Promise<T>;
}

export const ZERO_ADDRESS = new Uint8Array(32);

export class Transaction {
  protected registry: Registry;

  constructor(protected api: GearApi, types: Record<string, any>) {
    this.registry = new TypeRegistry();

    this.registry.setKnownTypes({ types });
    this.registry.register(types);
  }

  private sendTx(
    tx: SubmittableExtrinsic<'promise', ISubmittableResult>,
    account: IKeyringPair | string,
    signerOptions: Partial<SignerOptions>,
  ): Promise<{ msgId: string; blockHash: string }> {
    return new Promise((resolve, reject) =>
      tx
        .signAndSend(account, signerOptions, ({ events, status }) => {
          if (status.isInBlock) {
            let msgId: string;

            events.forEach(({ event }) => {
              const { method, section, data } = event;
              if (method === 'MessageQueued' && section === 'gear') {
                msgId = (data as MessageQueuedData).id.toHex();
              } else if (method === 'ExtrinsicSuccess') {
                resolve({ msgId, blockHash: status.asInBlock.toHex() });
              } else if (method === 'ExtrinsicFailed') {
                reject(this.api.getExtrinsicFailedError(event));
              }
            });
          }
        })
        .catch((error) => {
          reject(error.message);
        }),
    );
  }

  private async listenToUserMessageSentEvents(from: string, to: string) {
    const subject = new ReplaySubject<UserMessageSentData>(5);
    const unsub = await this.api.gearEvents.subscribeToGearEvent('UserMessageSent', ({ data }) => {
      if (data.message.source.eq(from) && data.message.destination.eq(to)) {
        subject.next(data);
      }
    });

    return { unsub, subject };
  }

  private async waitForReply(subject: ReplaySubject<UserMessageSentData>, msgId: string): Promise<HexString> {
    return new Promise<HexString>((resolve, reject) => {
      subject.subscribe(({ message }) => {
        if (message.details.isSome) {
          if (message.details.unwrap().to.eq(msgId)) {
            if (!message.details.unwrap().code.isSuccess) {
              reject(message.payload.toString());
            } else {
              resolve(message.payload.toHex());
            }
          }
        }
      });
    });
  }

  protected async submitMsg<T>(
    programId: HexString,
    payload: any,
    responseType: string,
    account: string | IKeyringPair,
    signerOptions: Partial<SignerOptions> = {},
    value: number | string | bigint = 0,
  ): Promise<{ msgId: string; blockHash: string; response: () => Promise<T> }> {
    const addressHex = decodeAddress(typeof account === 'string' ? account : account.address);

    const gasLimit = await this.api.program.calculateGas.handle(addressHex, programId, payload, value, false);

    const tx = this.api.message.send({
      destination: programId,
      payload,
      gasLimit: gasLimit.min_limit,
    });

    const { unsub, subject } = await this.listenToUserMessageSentEvents(programId, addressHex);

    const { msgId, blockHash } = await this.sendTx(tx, account, signerOptions);

    return {
      msgId,
      blockHash,
      response: () =>
        this.waitForReply(subject, msgId).then((reply) => {
          unsub();
          subject.complete();
          return this.registry.createType<any>(responseType, reply)[1].toJSON() as T;
        }),
    };
  }

  protected async uploadProgram(
    code: Uint8Array | Buffer,
    payload: any,
    account: string | IKeyringPair,
    signerOptions: Partial<SignerOptions> = {},
    value = 0,
  ) {
    const addressHex = decodeAddress(typeof account === 'string' ? account : account.address);

    const gas = await this.api.program.calculateGas.initUpload(addressHex, code, payload, value, true);

    const { extrinsic, programId } = this.api.program.upload({ code, gasLimit: gas.min_limit, initPayload: payload });

    const { unsub, subject } = await this.listenToUserMessageSentEvents(programId, addressHex);

    const { msgId, blockHash } = await this.sendTx(extrinsic, account, signerOptions);

    return {
      programId,
      msgId,
      blockHash,
      response: () =>
        this.waitForReply(subject, msgId).then(() => {
          unsub();
          subject.complete();
        }),
    };
  }
}

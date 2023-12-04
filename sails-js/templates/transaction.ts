import { GearApi, HexString, MessageQueuedData, UserMessageSentData, decodeAddress } from '@gear-js/api';
import { SubmittableExtrinsic } from '@polkadot/api/types';
import { TypeRegistry } from '@polkadot/types';
import { Codec, IKeyringPair, ISubmittableResult } from '@polkadot/types/types';
import { Registry } from '@polkadot/types-codec/types/registry';
import { ReplaySubject } from 'rxjs';

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
  ): Promise<string> {
    return new Promise((resolve, reject) =>
      tx
        .signAndSend(account, ({ events, status }) => {
          if (status.isInBlock) {
            let msgId: string;

            events.forEach(({ event }) => {
              const { method, section, data } = event;
              if (method === 'MessageQueued' && section === 'Gear') {
                msgId = (data as MessageQueuedData).id.toHex();
              } else if (section === 'System' && method === 'ExtrinsicSuccess') {
                resolve(msgId);
              } else if (section === 'System' && method === 'ExtrinsicFailed') {
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

  private async waitForReply(subject: ReplaySubject<UserMessageSentData>, msgId: string) {
    return new Promise<string>((resolve, reject) => {
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

  protected async submitMsgAndWaitForReply<Out extends Codec = Codec>(
    programId: HexString,
    payload: any,
    account: string | IKeyringPair,
    outputType: string,
  ): Promise<Out> {
    const addressHex = decodeAddress(typeof account === 'string' ? account : account.address);

    const gasLimit = await this.api.program.calculateGas.handle(addressHex, programId, payload, 0, false);

    const tx = this.api.message.send({
      destination: programId,
      payload,
      gasLimit: gasLimit.min_limit,
    });

    const { unsub, subject } = await this.listenToUserMessageSentEvents(addressHex, programId);

    const msgId = await this.sendTx(tx, account);

    const replyPayload = await this.waitForReply(subject, msgId);

    unsub();

    return this.registry.createType<Out>(outputType, replyPayload);
  }
}

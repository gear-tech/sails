import { GearApi } from '@gear-js/api';
import { Transaction } from './transaction.js';
import { IKeyringPair } from '@polkadot/types/types';

export type ThisThatSvcAppTupleStruct = [boolean];

export interface ThisThatSvcAppDoThatParam {
  p1: number | string | bigint;
  p2: string;
  p3: ThisThatSvcAppManyVariants;
}

export type ThisThatSvcAppManyVariants =
  | { One: null }
  | { Two: number | string | bigint }
  | { Three: number | string | bigint | null }
  | { Four: { a: number | string | bigint; b: number | string | bigint | null } }
  | { Five: [string, number | string | bigint] }
  | { Six: Array<Record<string, number | string | bigint>> };

export interface SimpleStruct {
  a: { ok: string } | { err: number | string | bigint };
  b: number | string | bigint;
}

export type SimpleEnum = 'One' | 'Two' | 'Three';

export class Service extends Transaction {
  constructor(api: GearApi, public programId: `0x${string}`) {
    const types: Record<string, any> = {
      ThisThatSvcAppTupleStruct: '(bool)',
      ThisThatSvcAppDoThatParam: { p1: 'u32', p2: 'String', p3: 'ThisThatSvcAppManyVariants' },
      ThisThatSvcAppManyVariants: {
        _enum: {
          One: 'Null',
          Two: 'u32',
          Three: 'Option<u32>',
          Four: { a: 'u32', b: 'Option<u16>' },
          Five: '(String, u32)',
          Six: '[BTreeMap<String, u32>; 3]',
        },
      },
      SimpleStruct: { a: 'Result<String, u32>', b: 'u32' },
      SimpleEnum: { _enum: ['One', 'Two', 'Three'] },
    };
    super(api, types);
  }

  public async doThis(
    p1: number | string | bigint,
    p2: string,
    p3: [string | null, number | string | bigint],
    p4: ThisThatSvcAppTupleStruct,
    account: `0x${string}` | IKeyringPair,
  ): Promise<[string, number | string | bigint]> {
    const payload = [
      ...this.registry.createType('String', 'DoThis/').toU8a(),
      ...this.registry.createType('u32', p1).toU8a(),
      ...this.registry.createType('String', p2).toU8a(),
      ...this.registry.createType('(Option<String>, u8)', p3).toU8a(),
      ...this.registry.createType('ThisThatSvcAppTupleStruct', p4).toU8a(),
    ];
    const replyPayloadBytes = await this.submitMsgAndWaitForReply(this.programId, payload, account);
    const result = this.registry.createType('(String, u32)', replyPayloadBytes);
    return result.toJSON() as [string, number | string | bigint];
  }

  public async doThat(
    param: ThisThatSvcAppDoThatParam,
    account: `0x${string}` | IKeyringPair,
  ): Promise<{ ok: [string, number | string | bigint] } | { err: [string] }> {
    const payload = [
      ...this.registry.createType('String', 'DoThat/').toU8a(),
      ...this.registry.createType('ThisThatSvcAppDoThatParam', param).toU8a(),
    ];
    const replyPayloadBytes = await this.submitMsgAndWaitForReply(this.programId, payload, account);
    const result = this.registry.createType('Result<(String, u32), (String)>', replyPayloadBytes);
    return result.toJSON() as { ok: [string, number | string | bigint] } | { err: [string] };
  }

  public async this(): Promise<number | string | bigint> {
    const payload = this.registry.createType('String', 'This/').toU8a();
    const stateBytes = await this.api.programState.read({ programId: this.programId, payload });
    const result = this.registry.createType('u32', stateBytes);
    return result.toBigInt() as number | string | bigint;
  }

  public async that(): Promise<{ ok: string } | { err: string }> {
    const payload = this.registry.createType('String', 'That/').toU8a();
    const stateBytes = await this.api.programState.read({ programId: this.programId, payload });
    const result = this.registry.createType('Result<String, String>', stateBytes);
    return result.toJSON() as { ok: string } | { err: string };
  }
}

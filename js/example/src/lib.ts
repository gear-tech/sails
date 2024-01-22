import { GearApi } from '@gear-js/api';
import { Transaction } from './transaction.js';
import { IKeyringPair } from '@polkadot/types/types';

export type Alias = bigint;

export type OptionAlias = null | number;

export type ResultAlias = { ok: number } | { err: string };

export type VecAlias = Array<number>;

export type ThisThatSvcAppTupleStruct = [boolean];

export interface ThisThatSvcAppDoThatParam {
  p1: bigint;
  p2: string;
  p3: ThisThatSvcAppManyVariants;
}

export type ThisThatSvcAppManyVariants = 
  | { One: null }
  | { Two: bigint }
  | { Three: null | bigint }
  | { Four: { a: bigint; b: null | number } }
  | { Five: [string, bigint] }
  | { Six: [bigint] };

export class Service extends Transaction {
  constructor(api: GearApi, public programId: `0x${string}`) {
    const types: Record<string, any> = {
      Alias: "u32",
      OptionAlias: "Option<u8>",
      ResultAlias: "Result<u8, string>",
      VecAlias: "Vec<u8>",
      ThisThatSvcAppTupleStruct: "(bool)",
      ThisThatSvcAppDoThatParam: {"p1":"u32","p2":"str","p3":"ThisThatSvcAppManyVariants"},
      ThisThatSvcAppManyVariants: {"_enum":{"One":null,"Two":"u32","Three":"Option<u32>","Four":{"a":"u32","b":"Option<u16>"},"Five":"(str, u32)","Six":"(u32)"}},
    }
    super(api, types);
  }

  public async doThis(p1: bigint, p2: string, p3: [null | string, number], p4: ThisThatSvcAppTupleStruct, account: `0x${string}` | IKeyringPair): Promise<[string, bigint]> {
    const payload = [
      ...this.registry.createType('String', 'DoThis/').toU8a(),
      ...this.registry.createType('u32', p1).toU8a(),
      ...this.registry.createType('str', p2).toU8a(),
      ...this.registry.createType('(Option<str>, u8)', p3).toU8a(),
      ...this.registry.createType('ThisThatSvcAppTupleStruct', p4).toU8a(),
    ];
    const replyPayloadBytes = await this.submitMsgAndWaitForReply(
      this.programId,
      payload,
      account,
    );
    const result = this.registry.createType('(str, u32)', replyPayloadBytes);
    return result.toJSON() as [string, bigint];
  }

  public async doThat(param: ThisThatSvcAppDoThatParam, account: `0x${string}` | IKeyringPair): Promise<{ ok: [string, bigint] } | { err: [string] }> {
    const payload = [
      ...this.registry.createType('String', 'DoThat/').toU8a(),
      ...this.registry.createType('ThisThatSvcAppDoThatParam', param).toU8a(),
    ];
    const replyPayloadBytes = await this.submitMsgAndWaitForReply(
      this.programId,
      payload,
      account,
    );
    const result = this.registry.createType('Result<(str, u32), (str)>', replyPayloadBytes);
    return result.toJSON() as { ok: [string, bigint] } | { err: [string] };
  }

  public async this(): Promise<bigint> {
    const payload = this.registry.createType('String', 'This/').toU8a();
    const stateBytes = await this.api.programState.read({ programId: this.programId, payload});
    const result = this.registry.createType('u32', stateBytes);
    return result.toBigInt() as bigint;
  }

  public async that(): Promise<{ ok: string } | { err: string }> {
    const payload = this.registry.createType('String', 'That/').toU8a();
    const stateBytes = await this.api.programState.read({ programId: this.programId, payload});
    const result = this.registry.createType('Result<str, str>', stateBytes);
    return result.toJSON() as { ok: string } | { err: string };
  }
}
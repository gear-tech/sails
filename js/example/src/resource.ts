import { GearApi, decodeAddress } from '@gear-js/api';
import { TypeRegistry } from '@polkadot/types';
import { TransactionBuilder } from 'sails-js';
import { u8aToHex, compactFromU8aLim } from '@polkadot/util';

const ZERO_ADDRESS = u8aToHex(new Uint8Array(32));

export type Error = "notAuthorized" | "zeroResourceId" | "resourceAlreadyExists" | "resourceNotFound" | "wrongResourceType" | "partNotFound";

export type Resource = 
  | { basic: BasicResource }
  | { slot: SlotResource }
  | { composed: ComposedResource };

export interface BasicResource {
  src: string;
  thumb: string | null;
  metadata_uri: string;
}

export interface SlotResource {
  src: string;
  thumb: string;
  metadata_uri: string;
  base: `0x${string}` | Uint8Array;
  slot: number | string;
}

export interface ComposedResource {
  src: string;
  thumb: string;
  metadata_uri: string;
  base: `0x${string}` | Uint8Array;
  parts: Array<number | string>;
}

export class RmrkResource {
  private registry: TypeRegistry;
  constructor(public api: GearApi, public programId?: `0x${string}`) {
    const types: Record<string, any> = {
      Error: {"_enum":["NotAuthorized","ZeroResourceId","ResourceAlreadyExists","ResourceNotFound","WrongResourceType","PartNotFound"]},
      Resource: {"_enum":{"Basic":"BasicResource","Slot":"SlotResource","Composed":"ComposedResource"}},
      BasicResource: {"src":"String","thumb":"Option<String>","metadata_uri":"String"},
      SlotResource: {"src":"String","thumb":"String","metadata_uri":"String","base":"[u8;32]","slot":"u32"},
      ComposedResource: {"src":"String","thumb":"String","metadata_uri":"String","base":"[u8;32]","parts":"Vec<u32>"},
    }

    this.registry = new TypeRegistry();
    this.registry.setKnownTypes({ types });
    this.registry.register(types);
  }

  newCtorFromCode(code: Uint8Array | Buffer): TransactionBuilder<null> {
    const builder = new TransactionBuilder<null>(
      this.api,
      this.registry,
      'upload_program',
      'New',
      'String',
      'String',
      code,
    );

    this.programId = builder.programId;
    return builder;
  }

  newCtorFromCodeId(codeId: `0x${string}`) {
    const builder = new TransactionBuilder<null>(
      this.api,
      this.registry,
      'create_program',
      'New',
      'String',
      'String',
      codeId,
    );

    this.programId = builder.programId;
    return builder;
  }

  public addPartToResource(resource_id: number | string, part_id: number | string): TransactionBuilder<{ ok: number | string } | { err: Error }> {
    return new TransactionBuilder<{ ok: number | string } | { err: Error }>(
      this.api,
      this.registry,
      'send_message',
      ['AddPartToResource', resource_id, part_id],
      '(String, u8, u32)',
      'Result<u32, Error>',
      this.programId
    );
  }

  public addResourceEntry(resource_id: number | string, resource: Resource): TransactionBuilder<{ ok: [number | string, Resource] } | { err: Error }> {
    return new TransactionBuilder<{ ok: [number | string, Resource] } | { err: Error }>(
      this.api,
      this.registry,
      'send_message',
      ['AddResourceEntry', resource_id, resource],
      '(String, u8, Resource)',
      'Result<(u8, Resource), Error>',
      this.programId
    );
  }

  public async resource(resource_id: number | string, originAddress: string, value?: number | string | bigint, atBlock?: `0x${string}`): Promise<{ ok: Resource } | { err: Error }> {
    const payload = this.registry.createType('(String, u8)', ['Resource', resource_id]).toU8a();
    const reply = await this.api.message.calculateReply({
      destination: this.programId,
      origin: decodeAddress(originAddress),
      payload,
      value: value || 0,
      gasLimit: this.api.blockGasLimit.toBigInt(),
      at: atBlock || null,
    });
    const result = this.registry.createType('(String, Result<Resource, Error>)', reply.payload);
    return result[1].toJSON() as unknown as { ok: Resource } | { err: Error };
  }

  public subscribeToResourceAddedEvent(callback: (data: { resource_id: number | string }) => void | Promise<void>): Promise<() => void> {
    return this.api.gearEvents.subscribeToGearEvent('UserMessageSent', ({ data: { message } }) => {;
      if (!message.source.eq(this.programId) || !message.destination.eq(ZERO_ADDRESS)) {
        return;
      }

      const payload = message.payload.toU8a();
      const [offset, limit] = compactFromU8aLim(payload);
      const name = this.registry.createType('String', payload.subarray(offset, limit)).toString();
      if (name === 'ResourceAdded') {
        callback(this.registry.createType('(String, {"resource_id":"u8"})', message.payload)[1].toJSON() as { resource_id: number | string });
      }
    });
  }

  public subscribeToPartAddedEvent(callback: (data: { resource_id: number | string; part_id: number | string }) => void | Promise<void>): Promise<() => void> {
    return this.api.gearEvents.subscribeToGearEvent('UserMessageSent', ({ data: { message } }) => {;
      if (!message.source.eq(this.programId) || !message.destination.eq(ZERO_ADDRESS)) {
        return;
      }

      const payload = message.payload.toU8a();
      const [offset, limit] = compactFromU8aLim(payload);
      const name = this.registry.createType('String', payload.subarray(offset, limit)).toString();
      if (name === 'PartAdded') {
        callback(this.registry.createType('(String, {"resource_id":"u8","part_id":"u32"})', message.payload)[1].toJSON() as { resource_id: number | string; part_id: number | string });
      }
    });
  }
}
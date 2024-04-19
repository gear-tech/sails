import { GearApi, decodeAddress } from '@gear-js/api';
import { TypeRegistry } from '@polkadot/types';
import { TransactionBuilder } from 'sails-js';

export type Error = "partIdCantBeZero" | "badConfig" | "partAlreadyExists" | "zeroLengthPassed" | "partDoesNotExist" | "wrongPartFormat" | "notAllowedToCall";

export type Part = 
  | { fixed: FixedPart }
  | { slot: SlotPart };

export interface FixedPart {
  z: number | string | null;
  metadata_uri: string;
}

export interface SlotPart {
  equippable: Array<`0x${string}` | Uint8Array>;
  z: number | string | null;
  metadata_uri: string;
}

export class RmrkCatalog {
  private registry: TypeRegistry;
  constructor(public api: GearApi, public programId?: `0x${string}`) {
    const types: Record<string, any> = {
      Error: {"_enum":["PartIdCantBeZero","BadConfig","PartAlreadyExists","ZeroLengthPassed","PartDoesNotExist","WrongPartFormat","NotAllowedToCall"]},
      Part: {"_enum":{"Fixed":"FixedPart","Slot":"SlotPart"}},
      FixedPart: {"z":"Option<u32>","metadata_uri":"String"},
      SlotPart: {"equippable":"Vec<[u8;32]>","z":"Option<u32>","metadata_uri":"String"},
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

  public addEquippables(part_id: number | string, collection_ids: Array<`0x${string}` | Uint8Array>): TransactionBuilder<{ ok: [number | string, Array<`0x${string}` | Uint8Array>] } | { err: Error }> {
    return new TransactionBuilder<{ ok: [number | string, Array<`0x${string}` | Uint8Array>] } | { err: Error }>(
      this.api,
      this.registry,
      'send_message',
      ['AddEquippables', part_id, collection_ids],
      '(String, u32, Vec<[u8;32]>)',
      'Result<(u32, Vec<[u8;32]>), Error>',
      this.programId
    );
  }

  public addParts(parts: Record<number | string, Part>): TransactionBuilder<{ ok: Record<number | string, Part> } | { err: Error }> {
    return new TransactionBuilder<{ ok: Record<number | string, Part> } | { err: Error }>(
      this.api,
      this.registry,
      'send_message',
      ['AddParts', parts],
      '(String, BTreeMap<u32, Part>)',
      'Result<BTreeMap<u32, Part>, Error>',
      this.programId
    );
  }

  public removeEquippable(part_id: number | string, collection_id: `0x${string}` | Uint8Array): TransactionBuilder<{ ok: [number | string, `0x${string}` | Uint8Array] } | { err: Error }> {
    return new TransactionBuilder<{ ok: [number | string, `0x${string}` | Uint8Array] } | { err: Error }>(
      this.api,
      this.registry,
      'send_message',
      ['RemoveEquippable', part_id, collection_id],
      '(String, u32, [u8;32])',
      'Result<(u32, [u8;32]), Error>',
      this.programId
    );
  }

  public removeParts(part_ids: Array<number | string>): TransactionBuilder<{ ok: Array<number | string> } | { err: Error }> {
    return new TransactionBuilder<{ ok: Array<number | string> } | { err: Error }>(
      this.api,
      this.registry,
      'send_message',
      ['RemoveParts', part_ids],
      '(String, Vec<u32>)',
      'Result<Vec<u32>, Error>',
      this.programId
    );
  }

  public resetEquippables(part_id: number | string): TransactionBuilder<{ ok: null } | { err: Error }> {
    return new TransactionBuilder<{ ok: null } | { err: Error }>(
      this.api,
      this.registry,
      'send_message',
      ['ResetEquippables', part_id],
      '(String, u32)',
      'Result<Null, Error>',
      this.programId
    );
  }

  public setEquippablesToAll(part_id: number | string): TransactionBuilder<{ ok: null } | { err: Error }> {
    return new TransactionBuilder<{ ok: null } | { err: Error }>(
      this.api,
      this.registry,
      'send_message',
      ['SetEquippablesToAll', part_id],
      '(String, u32)',
      'Result<Null, Error>',
      this.programId
    );
  }

  public async equippable(part_id: number | string, collection_id: `0x${string}` | Uint8Array, originAddress: string, value?: number | string | bigint, atBlock?: `0x${string}`): Promise<{ ok: boolean } | { err: Error }> {
    const payload = this.registry.createType('(String, u32, [u8;32])', ['Equippable', part_id, collection_id]).toU8a();
    const reply = await this.api.message.calculateReply({
      destination: this.programId,
      origin: decodeAddress(originAddress),
      payload,
      value: value || 0,
      gasLimit: this.api.blockGasLimit.toBigInt(),
      at: atBlock || null,
    });
    const result = this.registry.createType('(String, Result<bool, Error>)', reply.payload);
    return result[1].toJSON() as unknown as { ok: boolean } | { err: Error };
  }

  public async part(part_id: number | string, originAddress: string, value?: number | string | bigint, atBlock?: `0x${string}`): Promise<Part | null> {
    const payload = this.registry.createType('(String, u32)', ['Part', part_id]).toU8a();
    const reply = await this.api.message.calculateReply({
      destination: this.programId,
      origin: decodeAddress(originAddress),
      payload,
      value: value || 0,
      gasLimit: this.api.blockGasLimit.toBigInt(),
      at: atBlock || null,
    });
    const result = this.registry.createType('(String, Option<Part>)', reply.payload);
    return result[1].toJSON() as unknown as Part | null;
  }
}
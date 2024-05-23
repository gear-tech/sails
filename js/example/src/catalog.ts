import { GearApi, decodeAddress } from '@gear-js/api';
import { TypeRegistry } from '@polkadot/types';
import { TransactionBuilder } from 'sails-js';

export type Error = "partIdCantBeZero" | "badConfig" | "partAlreadyExists" | "zeroLengthPassed" | "partDoesNotExist" | "wrongPartFormat" | "notAllowedToCall";

export type Part = 
  | { fixed: FixedPart }
  | { slot: SlotPart };

export interface FixedPart {
  z: number | null;
  metadata_uri: string;
}

export interface SlotPart {
  equippable: Array<string>;
  z: number | null;
  metadata_uri: string;
}

export class RmrkCatalog {
  public readonly registry: TypeRegistry;
  public readonly service: Service;

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

    this.service = new Service(this);
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
}

export class Service {
  constructor(private _program: RmrkCatalog) {}

  public addEquippables(part_id: number, collection_ids: Array<string>): TransactionBuilder<{ ok: [number, Array<string>] } | { err: Error }> {
    if (!this._program.programId) throw new Error('Program ID is not set');
    return new TransactionBuilder<{ ok: [number, Array<string>] } | { err: Error }>(
      this._program.api,
      this._program.registry,
      'send_message',
      ['Service', 'AddEquippables', part_id, collection_ids],
      '(String, String, u32, Vec<[u8;32]>)',
      'Result<(u32, Vec<[u8;32]>), Error>',
      this._program.programId
    );
  }

  public addParts(parts: Record<number, Part>): TransactionBuilder<{ ok: Record<number, Part> } | { err: Error }> {
    if (!this._program.programId) throw new Error('Program ID is not set');
    return new TransactionBuilder<{ ok: Record<number, Part> } | { err: Error }>(
      this._program.api,
      this._program.registry,
      'send_message',
      ['Service', 'AddParts', parts],
      '(String, String, BTreeMap<u32, Part>)',
      'Result<BTreeMap<u32, Part>, Error>',
      this._program.programId
    );
  }

  public removeEquippable(part_id: number, collection_id: string): TransactionBuilder<{ ok: [number, string] } | { err: Error }> {
    if (!this._program.programId) throw new Error('Program ID is not set');
    return new TransactionBuilder<{ ok: [number, string] } | { err: Error }>(
      this._program.api,
      this._program.registry,
      'send_message',
      ['Service', 'RemoveEquippable', part_id, collection_id],
      '(String, String, u32, [u8;32])',
      'Result<(u32, [u8;32]), Error>',
      this._program.programId
    );
  }

  public removeParts(part_ids: Array<number>): TransactionBuilder<{ ok: Array<number> } | { err: Error }> {
    if (!this._program.programId) throw new Error('Program ID is not set');
    return new TransactionBuilder<{ ok: Array<number> } | { err: Error }>(
      this._program.api,
      this._program.registry,
      'send_message',
      ['Service', 'RemoveParts', part_ids],
      '(String, String, Vec<u32>)',
      'Result<Vec<u32>, Error>',
      this._program.programId
    );
  }

  public resetEquippables(part_id: number): TransactionBuilder<{ ok: null } | { err: Error }> {
    if (!this._program.programId) throw new Error('Program ID is not set');
    return new TransactionBuilder<{ ok: null } | { err: Error }>(
      this._program.api,
      this._program.registry,
      'send_message',
      ['Service', 'ResetEquippables', part_id],
      '(String, String, u32)',
      'Result<Null, Error>',
      this._program.programId
    );
  }

  public setEquippablesToAll(part_id: number): TransactionBuilder<{ ok: null } | { err: Error }> {
    if (!this._program.programId) throw new Error('Program ID is not set');
    return new TransactionBuilder<{ ok: null } | { err: Error }>(
      this._program.api,
      this._program.registry,
      'send_message',
      ['Service', 'SetEquippablesToAll', part_id],
      '(String, String, u32)',
      'Result<Null, Error>',
      this._program.programId
    );
  }

  public async equippable(part_id: number, collection_id: string, originAddress: string, value?: number | string | bigint, atBlock?: `0x${string}`): Promise<{ ok: boolean } | { err: Error }> {
    const payload = this._program.registry.createType('(String, String, u32, [u8;32])', ['Service', 'Equippable', part_id, collection_id]).toHex();
    const reply = await this._program.api.message.calculateReply({
      destination: this._program.programId,
      origin: decodeAddress(originAddress),
      payload,
      value: value || 0,
      gasLimit: this._program.api.blockGasLimit.toBigInt(),
      at: atBlock || null,
    });
    const result = this._program.registry.createType('(String, String, Result<bool, Error>)', reply.payload);
    return result[2].toJSON() as unknown as { ok: boolean } | { err: Error };
  }

  public async part(part_id: number, originAddress: string, value?: number | string | bigint, atBlock?: `0x${string}`): Promise<Part | null> {
    const payload = this._program.registry.createType('(String, String, u32)', ['Service', 'Part', part_id]).toHex();
    const reply = await this._program.api.message.calculateReply({
      destination: this._program.programId,
      origin: decodeAddress(originAddress),
      payload,
      value: value || 0,
      gasLimit: this._program.api.blockGasLimit.toBigInt(),
      at: atBlock || null,
    });
    const result = this._program.registry.createType('(String, String, Option<Part>)', reply.payload);
    return result[2].toJSON() as unknown as Part | null;
  }
}
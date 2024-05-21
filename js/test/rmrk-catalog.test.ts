import { GearApi, HexString, decodeAddress } from '@gear-js/api';
import { KeyringPair } from '@polkadot/keyring/types';
import { waitReady } from '@polkadot/wasm-crypto';
import { Keyring } from '@polkadot/api';
import { readFileSync } from 'fs';

import { Sails } from '../lib';
import { Program } from './rmrk-catalog/lib';

let sails: Sails;
let api: GearApi;
let alice: KeyringPair;
let aliceRaw: HexString;
let code: Buffer;

const IDL_PATH = '../examples/rmrk/catalog/wasm/rmrk-catalog.idl';
const CATALOG_WASM_PATH = '../target/wasm32-unknown-unknown/release/rmrk_catalog.opt.wasm';

beforeAll(async () => {
  sails = await Sails.new();
  api = await GearApi.create({ providerAddress: 'ws://127.0.0.1:9944' });
  await waitReady();
  const keyring = new Keyring({ type: 'sr25519' });
  alice = keyring.addFromUri('//Alice');
  aliceRaw = decodeAddress(alice.address);
  code = readFileSync(CATALOG_WASM_PATH);
});

afterAll(async () => {
  await api.disconnect();
  await new Promise((resolve) => {
    setTimeout(resolve, 2000);
  });
});

describe('RMRK catalog', () => {
  test('parse catalog idl', () => {
    const idl = readFileSync(IDL_PATH, 'utf-8');
    sails.parseIdl(idl);
  });

  test('upload catalog', async () => {
    sails.setApi(api);
    const transaction = await sails.ctors.New.fromCode(code)
      .withAccount(alice)
      .withGas(api.blockGasLimit.toBigInt() / 2n);
    const { response } = await transaction.signAndSend();
    await response();
  });

  test('add parts func', async () => {
    expect(sails.programId).toBeDefined();
    expect(sails.services).toHaveProperty('RmrkCatalog');

    const transaction = await sails.services.RmrkCatalog.functions
      .AddParts({
        1: { Fixed: { z: null, metadata_uri: 'foo' } },
      })
      .withAccount(alice)
      .calculateGas();
    const { response, blockHash, txHash, msgId } = await transaction.signAndSend();
    const result = await response();

    expect(blockHash).toBeDefined();
    expect(msgId).toBeDefined();
    expect(txHash).toBeDefined();

    expect(result).toEqual({
      ok: {
        '1': { fixed: { z: null, metadata_uri: 'foo' } },
      },
    });
  });

  test('read parts', async () => {
    const result = await sails.services.RmrkCatalog.queries.Part(alice.address, null, null, 1);

    expect(result).toEqual({
      fixed: {
        metadata_uri: 'foo',
        z: null,
      },
    });
  });
});

let program: Program;

describe('RMRK generated', () => {
  let programCreated = false;

  test('create program', async () => {
    program = new Program(api);

    expect(program.rmrkCatalog).toHaveProperty('addParts');
    expect(program.rmrkCatalog).toHaveProperty('removeParts');
    expect(program.rmrkCatalog).toHaveProperty('addEquippables');
    expect(program.rmrkCatalog).toHaveProperty('removeEquippable');
    expect(program.rmrkCatalog).toHaveProperty('resetEquippables');
    expect(program.rmrkCatalog).toHaveProperty('setEquippablesToAll');
    expect(program.rmrkCatalog).toHaveProperty('part');
    expect(program.rmrkCatalog).toHaveProperty('equippable');

    const transaction = await program.newCtorFromCode(code).withAccount(alice).calculateGas();

    const { msgId, blockHash } = await transaction.signAndSend();

    expect(msgId).toBeDefined();
    expect(blockHash).toBeDefined();

    programCreated = true;
  });

  test('add parts', async () => {
    expect(programCreated).toBeTruthy();
    expect(program).toBeDefined();
    const transaction = await program.rmrkCatalog.addParts({
      1: { fixed: { z: null, metadata_uri: 'foo' } },
      2: { fixed: { z: 0, metadata_uri: 'bar' } },
      3: { slot: { z: 1, equippable: [aliceRaw], metadata_uri: 'baz' } },
    });

    await transaction.withAccount(alice).calculateGas();

    const result = await transaction.signAndSend();

    expect(result).toHaveProperty('msgId');
    expect(result).toHaveProperty('blockHash');
    expect(result).toHaveProperty('response');
    expect(result.msgId).toBeDefined();
    expect(result.blockHash).toBeDefined();

    const response = await result.response();

    expect(response).toHaveProperty('ok');
    if ('ok' in response) {
      expect(response.ok).toHaveProperty('2');
      expect(response.ok).toHaveProperty('3');
      expect(response.ok[2]).toHaveProperty('fixed');
      expect(response.ok[3]).toHaveProperty('slot');
      if ('fixed' in response.ok[2]) {
        expect(response.ok[2].fixed).toHaveProperty('z', 0);
        expect(response.ok[2].fixed).toHaveProperty('metadata_uri', 'bar');
      }
      if ('slot' in response.ok[3]) {
        expect(response.ok[3].slot).toHaveProperty('z', 1);
        expect(response.ok[3].slot).toHaveProperty('metadata_uri', 'baz');
        expect(response.ok[3].slot).toHaveProperty('equippable');
        expect(response.ok[3].slot.equippable).toHaveLength(1);
        expect(response.ok[3].slot.equippable[0]).toBe(aliceRaw);
      }
    }
  });

  test('remove parts', async () => {
    expect(programCreated).toBeTruthy();
    expect(program).toBeDefined();
    const transaction = await program.rmrkCatalog.removeParts([1]);

    await transaction.withAccount(alice).calculateGas();

    const result = await transaction.signAndSend();

    const response = await result.response();

    expect(response).toHaveProperty('ok');
    if ('ok' in response) {
      expect(response.ok).toHaveLength(1);
      expect(response.ok[0]).toBe(1);
    }
  });

  test('add equippables', async () => {
    expect(programCreated).toBeTruthy();
    expect(program).toBeDefined();
    const transaction = program.rmrkCatalog.addEquippables(3, [aliceRaw]);

    await transaction.withAccount(alice).calculateGas();

    const result = await transaction.signAndSend();

    const response = await result.response();

    expect(response).toHaveProperty('ok');
    if ('ok' in response) {
      expect(response.ok).toHaveLength(2);
      expect(response.ok[0]).toBe(3);
      expect(response.ok[1]).toHaveLength(1);
      expect(response.ok[1][0]).toBe(aliceRaw);
    }
  });

  test('remove equippable', async () => {
    expect(programCreated).toBeTruthy();
    expect(program).toBeDefined();
    const transaction = await program.rmrkCatalog.removeEquippable(3, aliceRaw);

    await transaction.withAccount(alice).calculateGas();

    const result = await transaction.signAndSend();

    const response = await result.response();

    expect(response).toHaveProperty('ok');
    if ('ok' in response) {
      expect(response.ok).toHaveLength(2);
      expect(response.ok[0]).toBe(3);
      expect(response.ok[1]).toBe(aliceRaw);
    }
  });

  test('reset equippables', async () => {
    expect(programCreated).toBeTruthy();
    expect(program).toBeDefined();
    const transaction = await program.rmrkCatalog.resetEquippables(3);

    await transaction.withAccount(alice).calculateGas();

    const result = await transaction.signAndSend();

    const response = await result.response();

    expect(response).toHaveProperty('ok');
    if ('ok' in response) {
      expect(response.ok).toBeNull();
    }
  });

  test('set equippables to all', async () => {
    expect(programCreated).toBeTruthy();
    expect(program).toBeDefined();
    const transaction = await program.rmrkCatalog.setEquippablesToAll(3);

    await transaction.withAccount(alice).calculateGas();

    const result = await transaction.signAndSend();

    const response = await result.response();

    expect(response).toHaveProperty('ok');
    if ('ok' in response) {
      expect(response.ok).toBeNull();
    }
  });

  test('read state: part', async () => {
    expect(programCreated).toBeTruthy();
    expect(program).toBeDefined();
    const result = await program.rmrkCatalog.part(2, aliceRaw);

    expect(result).toEqual({
      fixed: {
        metadata_uri: 'bar',
        z: 0,
      },
    });
  });
});

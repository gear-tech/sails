import { GearApi, HexString, MessageQueued, decodeAddress } from '@gear-js/api';
import { KeyringPair } from '@polkadot/keyring/types';
import { waitReady } from '@polkadot/wasm-crypto';
import { Keyring } from '@polkadot/api';
import { u8aToHex } from '@polkadot/util';

import { Sails } from '../lib';
import { readFileSync } from 'fs';
import { Program } from './lib';

let sails: Sails;
let api: GearApi;
let alice: KeyringPair;
let catalogId: HexString;
let aliceRaw: HexString;
let code: Buffer;

const IDL_PATH = '../examples/rmrk/catalog/wasm/rmrk-catalog.idl';
const CATALOG_WASM_PATH = '../target/wasm32-unknown-unknown/debug/rmrk_catalog.opt.wasm';

beforeAll(async () => {
  sails = await Sails.new();
  api = await GearApi.create({ providerAddress: 'ws://127.0.0.1:9944' });
  await waitReady();
  alice = new Keyring().addFromUri('//Alice', {}, 'sr25519');
  aliceRaw = decodeAddress(alice.address);
});

afterAll(async () => {
  await api.disconnect();
  await new Promise((resolve) => {
    setTimeout(resolve, 2000);
  });
});

describe('RMRK', () => {
  test('parse catalog idl', () => {
    const idl = readFileSync(IDL_PATH, 'utf-8');
    sails.parseIdl(idl);
  });

  test('upload catalog', async () => {
    code = readFileSync(CATALOG_WASM_PATH);

    const payload = sails.ctors.New.encodePayload();

    const gas = await api.program.calculateGas.initUpload(aliceRaw, code, payload);

    // TODO: replace u8aToHex(payload) with payload after next @gear-js/api release
    const { extrinsic, programId } = api.program.upload({
      code,
      gasLimit: gas.min_limit,
      initPayload: u8aToHex(payload),
    });

    await new Promise((resolve, reject) => {
      extrinsic.signAndSend(alice, ({ events, status }) => {
        if (status.isInBlock) {
          const success = events.find(({ event: { method } }) => method === 'ExtrinsicSuccess');
          if (success) {
            resolve(0);
          } else {
            const failed = events.find(({ event: { method } }) => method === 'ExtrinsicFailed');
            reject(api.getExtrinsicFailedError(failed.event).docs);
          }
        }
      });
    });

    catalogId = programId;
  });

  test('add parts func', async () => {
    expect(catalogId).toBeDefined();
    const payload = sails.functions.AddParts.encodePayload({
      1: { Fixed: { z: null, metadata_uri: 'foo' } },
    });

    const gas = await api.program.calculateGas.handle(aliceRaw, catalogId, payload);

    const extrinsic = api.message.send({ destination: catalogId, payload, gasLimit: gas.min_limit });

    const reply = api.message.listenToReplies(catalogId);

    let msgId = await new Promise<HexString>((resolve, reject) => {
      extrinsic.signAndSend(alice, ({ events, status }) => {
        if (status.isInBlock) {
          const success = events.find(({ event: { method } }) => method === 'ExtrinsicSuccess');
          if (success) {
            const msgQueued = events.find(({ event: { method } }) => method === 'MessageQueued').event as MessageQueued;
            resolve(msgQueued.data.id.toHex());
          } else {
            const failed = events.find(({ event: { method } }) => method === 'ExtrinsicFailed');
            reject(api.getExtrinsicFailedError(failed.event).docs);
          }
        }
      });
    });

    const replyMsg = await reply(msgId);

    expect(replyMsg).toBeDefined();

    const result = sails.functions.AddParts.decodeResult(replyMsg.message.payload);

    expect(result).toEqual({
      ok: {
        '1': { fixed: { z: null, metadata_uri: 'foo' } },
      },
    });
  });
});

let program: Program;

describe('RMRK generated', () => {
  test('create program', async () => {
    program = new Program(api);
    expect(program).toHaveProperty('newCtor');

    await program.newCtor(code, alice);

    expect(program).toHaveProperty('addParts');
    expect(program).toHaveProperty('removeParts');
    expect(program).toHaveProperty('addEquippables');
    expect(program).toHaveProperty('removeEquippable');
    expect(program).toHaveProperty('resetEquippables');
    expect(program).toHaveProperty('setEquippablesToAll');
    expect(program).toHaveProperty('part');
    expect(program).toHaveProperty('equippable');
    expect(program.programId).toBeDefined();
  });

  test('add parts', async () => {
    expect(program).toBeDefined();
    const result = await program.addParts(
      {
        1: { fixed: { z: null, metadata_uri: 'foo' } },
        2: { fixed: { z: 0, metadata_uri: 'bar' } },
        3: { slot: { z: 1, equippable: [aliceRaw], metadata_uri: 'baz' } },
      },
      alice,
    );
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
    expect(program).toBeDefined();
    const result = await program.removeParts([1], alice);

    const response = await result.response();

    expect(response).toHaveProperty('ok');
    if ('ok' in response) {
      expect(response.ok).toHaveLength(1);
      expect(response.ok[0]).toBe(1);
    }
  });

  test('add equippables', async () => {
    expect(program).toBeDefined();
    const result = await program.addEquippables(3, [aliceRaw], alice);

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
    expect(program).toBeDefined();
    const result = await program.removeEquippable(3, aliceRaw, alice);

    const response = await result.response();

    expect(response).toHaveProperty('ok');
    if ('ok' in response) {
      expect(response.ok).toHaveLength(2);
      expect(response.ok[0]).toBe(3);
      expect(response.ok[1]).toBe(aliceRaw);
    }
  });

  test('reset equippables', async () => {
    expect(program).toBeDefined();
    const result = await program.resetEquippables(3, alice);

    const response = await result.response();

    expect(response).toHaveProperty('ok');
    if ('ok' in response) {
      expect(response.ok).toBeNull();
    }
  });

  test('set equippables to all', async () => {
    expect(program).toBeDefined();
    const result = await program.setEquippablesToAll(3, alice);

    const response = await result.response();

    expect(response).toHaveProperty('ok');
    if ('ok' in response) {
      expect(response.ok).toBeNull();
    }
  });

  test.todo('read state');
});

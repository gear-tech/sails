import { GearApi, HexString, MessageQueued, decodeAddress } from '@gear-js/api';
import { KeyringPair } from '@polkadot/keyring/types';
import { waitReady } from '@polkadot/wasm-crypto';
import { Keyring } from '@polkadot/api';
import { readFileSync } from 'fs';

import { Sails } from '../lib';
import { Program } from './rmrk-resource/lib';

let sails: Sails;
let api: GearApi;
let alice: KeyringPair;
let aliceRaw: HexString;
let code: Buffer;
let resourceId: HexString;

const IDL_PATH = '../examples/rmrk/resource/wasm/rmrk-resource.idl';
const RESOURCE_WASM_PATH = '../target/wasm32-unknown-unknown/debug/rmrk_resource.opt.wasm';

beforeAll(async () => {
  sails = await Sails.new();
  api = await GearApi.create({ providerAddress: 'ws://127.0.0.1:9944' });
  await waitReady();
  alice = new Keyring().addFromUri('//Alice', {}, 'sr25519');
  aliceRaw = decodeAddress(alice.address);
  code = readFileSync(RESOURCE_WASM_PATH);
});

afterAll(async () => {
  await api.disconnect();
  await new Promise((resolve) => {
    setTimeout(resolve, 2000);
  });
});

let program: Program;

describe('RMRK resource', () => {
  test('parse resource idl', () => {
    const idl = readFileSync(IDL_PATH, 'utf-8');
    sails.parseIdl(idl);
  });

  test('upload resource', async () => {
    const payload = sails.ctors.New.encodePayload();
    const gas = await api.program.calculateGas.initUpload(aliceRaw, code, payload);

    const { extrinsic, programId } = api.program.upload({
      code,
      gasLimit: gas.min_limit,
      initPayload: payload,
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

    resourceId = programId;
  });

  test('AddResourceEntry func', async () => {
    expect(resourceId).toBeDefined();

    const payload = sails.services.Service.functions.AddResourceEntry.encodePayload(1, {
      basic: {
        src: 'src',
        thumb: null,
        metadata_uri: 'metadata_uri',
      },
    });

    const gas = await api.program.calculateGas.handle(aliceRaw, resourceId, payload);

    const extrinsic = api.message.send({ destination: resourceId, payload, gasLimit: gas.min_limit });

    let resourceAddedEvent: any;

    const unsub = await api.gearEvents.subscribeToGearEvent('UserMessageSent', (event) => {
      if (!event.data.message.source.eq(resourceId)) {
        return;
      }

      if (!sails.services.Service.events.ResourceAdded.is(event)) {
        return;
      }

      resourceAddedEvent = sails.services.Service.events.ResourceAdded.decode(event.data.message.payload.toU8a());
    });

    let [msgId, blockHash] = await new Promise<[HexString, HexString]>((resolve, reject) => {
      extrinsic.signAndSend(alice, ({ events, status }) => {
        if (status.isInBlock) {
          const success = events.find(({ event: { method } }) => method === 'ExtrinsicSuccess');
          if (success) {
            const msgQueued = events.find(({ event: { method } }) => method === 'MessageQueued')
              ?.event as MessageQueued;
            resolve([msgQueued.data.id.toHex(), status.asInBlock.toHex()]);
          } else {
            const failed = events.find(({ event: { method } }) => method === 'ExtrinsicFailed');
            reject(api.getExtrinsicFailedError(failed.event).docs);
          }
        }
      });
    });

    const replyMsg = await api.message.getReplyEvent(resourceId, msgId, blockHash);

    expect(replyMsg).toBeDefined();

    const result = sails.services.Service.functions.AddResourceEntry.decodeResult(replyMsg.data.message.payload);

    expect(result).toEqual({
      ok: [
        1,
        {
          basic: {
            src: 'src',
            thumb: null,
            metadata_uri: 'metadata_uri',
          },
        },
      ],
    });

    expect(resourceAddedEvent).toBeDefined();

    expect(resourceAddedEvent).toHaveProperty('resource_id', 1);

    unsub();
  });
});

describe('RMRK resource generated', () => {
  test('create program', async () => {
    program = new Program(api);

    const transaction = await program.newCtorFromCode(code);

    await transaction.withAccount(alice).calculateGas();
    const { msgId, response, blockHash } = await transaction.signAndSend();

    expect(msgId).toBeDefined();
    expect(blockHash).toBeDefined();
    await response();

    expect(program.service).toHaveProperty('addPartToResource');
    expect(program.service).toHaveProperty('addResourceEntry');
    expect(program.service).toHaveProperty('resource');
    expect(program.programId).toBeDefined();
  });

  test('add resource and listen to event', async () => {
    let resourceEvent;

    const unsub = await program.service.subscribeToResourceAddedEvent((data) => {
      resourceEvent = data;
    });

    const transaction = await program.service.addResourceEntry(1, {
      composed: {
        src: 'src',
        thumb: 'thumb',
        metadata_uri: 'metadata_uri',
        base: aliceRaw,
        parts: [],
      },
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
      expect(response.ok[0]).toBe(1);
      expect(response.ok[1]).toHaveProperty('composed');
      if ('composed' in response.ok[1]) {
        expect(response.ok[1].composed).toHaveProperty('src', 'src');
        expect(response.ok[1].composed).toHaveProperty('thumb', 'thumb');
        expect(response.ok[1].composed).toHaveProperty('metadata_uri', 'metadata_uri');
        expect(response.ok[1].composed).toHaveProperty('base', aliceRaw);
        expect(response.ok[1].composed).toHaveProperty('parts', []);
      }
    }

    expect(resourceEvent).toBeDefined();
    expect(resourceEvent).toHaveProperty('resource_id', 1);

    unsub();
  });
});

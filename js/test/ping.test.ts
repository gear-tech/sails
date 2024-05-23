import { GearApi, HexString, MessageQueued, VoucherIssued, decodeAddress } from '@gear-js/api';
import { KeyringPair } from '@polkadot/keyring/types';
import { waitReady } from '@polkadot/wasm-crypto';
import { Keyring } from '@polkadot/api';
import { readFileSync } from 'fs';

import { Sails } from '../lib';
import { Program } from './ping/lib';

let sails: Sails;
let api: GearApi;
let alice: KeyringPair;
let aliceRaw: HexString;
let charlie: KeyringPair;
let charlieRaw: HexString;
let code: Buffer;

const CATALOG_WASM_PATH = '../target/wasm32-unknown-unknown/release/ping.opt.wasm';

beforeAll(async () => {
  sails = await Sails.new();
  api = await GearApi.create({ providerAddress: 'ws://127.0.0.1:9944' });
  await waitReady();
  const keyring = new Keyring({ type: 'sr25519' });
  alice = keyring.addFromUri('//Alice');
  aliceRaw = decodeAddress(alice.address);
  charlie = keyring.addFromUri('//Charlie');
  charlieRaw = decodeAddress(charlie.address);
  code = readFileSync(CATALOG_WASM_PATH);
});

afterAll(async () => {
  await api.disconnect();
  await new Promise((resolve) => {
    setTimeout(resolve, 2000);
  });
});

describe('Ping', () => {
  let program: Program;

  test('create program', async () => {
    program = new Program(api);

    const transaction = await program.newCtorFromCode(code).withAccount(alice).calculateGas();

    const { msgId, blockHash, response } = await transaction.signAndSend();

    expect(msgId).toBeDefined();
    expect(blockHash).toBeDefined();

    await response();
  });

  test('ping', async () => {
    const transaction = await program.ping.ping('ping').withAccount(alice).calculateGas();

    const { msgId, blockHash, response } = await transaction.signAndSend();

    expect(msgId).toBeDefined();
    expect(blockHash).toBeDefined();

    const result = await response();

    expect(result).toHaveProperty('ok', 'pong');
  });
});

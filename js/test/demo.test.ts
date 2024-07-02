import { GearApi, HexString, MessageQueued, VoucherIssued, decodeAddress, generateCodeHash } from '@gear-js/api';
import { KeyringPair } from '@polkadot/keyring/types';
import { waitReady } from '@polkadot/wasm-crypto';
import { Keyring } from '@polkadot/api';
import { readFileSync } from 'fs';

import { Sails } from '../lib';
import { Program } from './demo/lib';

let sails: Sails;
let api: GearApi;
let alice: KeyringPair;
let aliceRaw: HexString;
let charlie: KeyringPair;
let charlieRaw: HexString;
let code: Buffer;
let codeId: HexString;

const DEMO_WASM_PATH = '../target/wasm32-unknown-unknown/release/demo.opt.wasm';

beforeAll(async () => {
  sails = await Sails.new();
  api = await GearApi.create({ providerAddress: 'ws://127.0.0.1:9944' });
  await waitReady();
  const keyring = new Keyring({ type: 'sr25519' });
  alice = keyring.addFromUri('//Alice');
  aliceRaw = decodeAddress(alice.address);
  charlie = keyring.addFromUri('//Charlie');
  charlieRaw = decodeAddress(charlie.address);
  code = readFileSync(DEMO_WASM_PATH);
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
    const transaction = await program.defaultCtorFromCode(code).withAccount(alice).calculateGas();
    codeId = generateCodeHash(code);

    const { msgId, blockHash, response } = await transaction.signAndSend();

    expect(msgId).toBeDefined();
    expect(blockHash).toBeDefined();

    await response();
  });

  test('ping', async () => {
    expect(program.programId).toBeDefined();
    const transaction = await program.pingPong.ping('ping').withAccount(alice).calculateGas();

    const { msgId, blockHash, response } = await transaction.signAndSend();

    expect(msgId).toBeDefined();
    expect(blockHash).toBeDefined();

    const result = await response();

    expect(result).toHaveProperty('ok', 'pong');
  });

  test('ping with voucher', async () => {
    expect(program.programId).toBeDefined();
    const { extrinsic, voucherId } = await api.voucher.issue(decodeAddress(charlie.address), 10 * 1e12);
    await new Promise((resolve, reject) =>
      extrinsic.signAndSend(alice, ({ events, status }) => {
        if (status.isInBlock) {
          const success = events.find((record) => record.event.method === 'ExtrinsicSuccess');
          if (success) {
            resolve(voucherId);
          } else {
            reject(new Error('Extrinisc failed'));
          }
        }
      }),
    );

    const transaction = await program.pingPong.ping('ping').withAccount(charlie).withVoucher(voucherId).calculateGas();
    const { response } = await transaction.signAndSend();

    const result = await response();

    expect(result).toHaveProperty('ok', 'pong');
  });
});

describe('Counter', () => {
  let program: Program;

  test('create program from code id', async () => {
    program = new Program(api);
    const transaction = await program.defaultCtorFromCodeId(codeId).withAccount(alice).calculateGas();

    const { msgId, blockHash, response } = await transaction.signAndSend();

    expect(msgId).toBeDefined();
    expect(blockHash).toBeDefined();

    await response();
  });

  test('add', async () => {
    const transaction = await program.counter.add(5).withAccount(alice).calculateGas();

    let addedEventData: number;

    const unsub = await program.counter.subscribeToAddedEvent((data) => {
      addedEventData = data;
    });

    const { msgId, blockHash, response } = await transaction.signAndSend();

    expect(msgId).toBeDefined();
    expect(blockHash).toBeDefined();

    const result = await response();

    expect(result).toBe(5);
    expect(addedEventData).toBe(5);

    unsub();
  });

  test('sub', async () => {
    const transaction = await program.counter.sub(3).withAccount(alice).calculateGas();

    let subtractedEventData: number;

    const unsub = await program.counter.subscribeToSubtractedEvent((data) => {
      subtractedEventData = data;
    });

    const { msgId, blockHash, response } = await transaction.signAndSend();

    expect(msgId).toBeDefined();
    expect(blockHash).toBeDefined();

    const result = await response();

    expect(result).toBe(2);
    expect(subtractedEventData).toBe(3);

    unsub();
  });

  test('query Value', async () => {
    const value = await program.counter.value(aliceRaw);

    expect(value).toBe(2);
  });
});

describe('Dog', () => {
  let program: Program;

  test('create program from code id', async () => {
    program = new Program(api);

    const transaction = await program.newCtorFromCodeId(codeId, null, [5, 5]).withAccount(alice).calculateGas();

    const { msgId, blockHash, response } = await transaction.signAndSend();

    expect(msgId).toBeDefined();
    expect(blockHash).toBeDefined();

    await response();
  });

  test('MakeSound', async () => {
    const transaction = await program.dog.makeSound().withAccount(alice).calculateGas();

    let barked: boolean;

    const unsub = await program.dog.subscribeToBarkedEvent((data) => {
      barked = true;
    });

    const { msgId, blockHash, response } = await transaction.signAndSend();

    expect(msgId).toBeDefined();
    expect(blockHash).toBeDefined();

    const result = await response();

    expect(result).toBe('Woof! Woof!');
    expect(barked).toBe(true);

    unsub();
  });

  test('Walk', async () => {
    const transaction = await program.dog.walk(5, 10).withAccount(alice).calculateGas();

    let walkedFrom: [number, number];
    let walkedTo: [number, number];

    const unsub = await program.dog.subscribeToWalkedEvent((data) => {
      walkedFrom = data.from;
      walkedTo = data.to;
    });

    const { msgId, blockHash, response } = await transaction.signAndSend();

    expect(msgId).toBeDefined();
    expect(blockHash).toBeDefined();

    const result = await response();

    expect(result).toBe(null);

    expect(walkedFrom).toEqual([5, 5]);
    expect(walkedTo).toEqual([10, 15]);

    unsub();
  });

  test('query Position', async () => {
    const position = await program.dog.position(aliceRaw);

    expect(position).toEqual([10, 15]);
  });
});

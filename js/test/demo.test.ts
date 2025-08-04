import { GearApi, HexString, decodeAddress, generateCodeHash } from '@gear-js/api';
import { KeyringPair } from '@polkadot/keyring/types';
import { waitReady } from '@polkadot/wasm-crypto';
import { Keyring } from '@polkadot/api';
import { readFileSync } from 'node:fs';

import { H256, NonZeroU32, NonZeroU8, ZERO_ADDRESS } from '..';
import { SailsProgram } from './demo/lib';

let api: GearApi;
let alice: KeyringPair;
let aliceRaw: HexString;
let charlie: KeyringPair;
let charlieRaw: HexString;
let code: Buffer;
let codeId: HexString;

const DEMO_WASM_PATH = 'test/demo/demo.wasm';

beforeAll(async () => {
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
  let program: SailsProgram;

  test('create program', async () => {
    program = new SailsProgram(api);
    const transaction = program.defaultCtorFromCode(code).withAccount(alice).withGas('max');
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

    expect(transaction.gasInfo).toBeDefined();
    expect(msgId).toBeDefined();
    expect(blockHash).toBeDefined();

    const result = await response();

    expect(result).toHaveProperty('ok', 'pong');
  });

  test('ping w/o specific gas', async () => {
    expect(program.programId).toBeDefined();
    const transaction = program.pingPong.ping('ping').withAccount(alice);

    const { msgId, blockHash, response } = await transaction.signAndSend();

    expect(transaction.gasInfo).toBeDefined();
    expect(msgId).toBeDefined();
    expect(blockHash).toBeDefined();

    const result = await response();

    expect(result).toHaveProperty('ok', 'pong');
  });

  test('ping w/ low gas should fail', async () => {
    expect(program.programId).toBeDefined();
    const transaction = program.pingPong.ping('ping').withAccount(alice).withGas(1000n);

    const { msgId, blockHash, response } = await transaction.signAndSend();

    expect(msgId).toBeDefined();
    expect(blockHash).toBeDefined();

    await expect(response()).rejects.toThrow('Message ran out of gas while executing.');
  });

  test('ping with voucher', async () => {
    expect(program.programId).toBeDefined();
    const { extrinsic, voucherId } = await api.voucher.issue(charlieRaw, 10 * 1e12);
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

    expect(transaction.gasInfo).toBeDefined();
    expect(result).toHaveProperty('ok', 'pong');
  });
});

describe('Counter', () => {
  let program: SailsProgram;

  test('create program from code id', async () => {
    program = new SailsProgram(api);
    const transaction = await program.defaultCtorFromCodeId(codeId).withAccount(alice).calculateGas();

    const { msgId, blockHash, response } = await transaction.signAndSend();

    expect(transaction.gasInfo).toBeDefined();
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

    expect(transaction.gasInfo).toBeDefined();
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

    expect(transaction.gasInfo).toBeDefined();
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
  let program: SailsProgram;

  test('create program from code id', async () => {
    program = new SailsProgram(api);

    const transaction = await program.newCtorFromCodeId(codeId, null, [5, 5]).withAccount(alice).calculateGas();

    const { msgId, blockHash, response } = await transaction.signAndSend();

    expect(transaction.gasInfo).toBeDefined();
    expect(msgId).toBeDefined();
    expect(blockHash).toBeDefined();

    await response();
  });

  test('MakeSound', async () => {
    const transaction = await program.dog.makeSound().withAccount(alice).calculateGas();

    let barked: boolean;

    const unsub = await program.dog.subscribeToBarkedEvent(() => {
      barked = true;
    });

    const { msgId, blockHash, response } = await transaction.signAndSend();

    expect(transaction.gasInfo).toBeDefined();
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

    expect(transaction.gasInfo).toBeDefined();
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

describe('ThisThat', () => {
  let program: SailsProgram;

  test('create program from code id', async () => {
    program = new SailsProgram(api);

    const transaction = await program.defaultCtorFromCodeId(codeId).withAccount(alice).calculateGas();

    const { msgId, blockHash, response } = await transaction.signAndSend();

    expect(transaction.gasInfo).toBeDefined();
    expect(msgId).toBeDefined();
    expect(blockHash).toBeDefined();

    await response();
  });

  test('doThis', async () => {
    const tx = await program.thisThat
      .doThis(1, 'a', [null, NonZeroU8(1)], [true])
      .withAccount(alice)
      .calculateGas();

    const { msgId, blockHash, response } = await tx.signAndSend();

    expect(tx.gasInfo).toBeDefined();
    expect(msgId).toBeDefined();
    expect(blockHash).toBeDefined();

    const result = await response();

    expect(result).toEqual(['a', 1]);
  });

  test('doThat', async () => {
    const tx = await program.thisThat
      .doThat({ p1: NonZeroU32(2), p2: ZERO_ADDRESS, p3: { Five: ['c', H256(ZERO_ADDRESS)] } })
      .withAccount(alice)
      .calculateGas();

    const { msgId, blockHash, response } = await tx.signAndSend();

    expect(tx.gasInfo).toBeDefined();
    expect(msgId).toBeDefined();
    expect(blockHash).toBeDefined();

    const decodedResult = await response();
    expect(decodedResult).toEqual({ ok: [ZERO_ADDRESS, 2, 'Five'] });
  });
});

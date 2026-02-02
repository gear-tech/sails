import { GearApi, HexString, decodeAddress, generateCodeHash } from '@gear-js/api';
import { KeyringPair } from '@polkadot/keyring/types';
import { waitReady } from '@polkadot/wasm-crypto';
import { Keyring } from '@polkadot/api';
import { readFileSync } from 'node:fs';

import { SailsProgram, H256, NonZeroU32, NonZeroU8, ZERO_ADDRESS } from '..';
import { SailsIdlParser } from 'sails-js-parser-v2';

let api: GearApi;
let alice: KeyringPair;
let charlie: KeyringPair;
let charlieRaw: HexString;
let code: Buffer;
let codeId: HexString;
let program: SailsProgram;
let programId: HexString;

const DEMO_WASM_PATH = '../target/wasm32-gear/release/demo.opt.wasm';
const DEMO_IDL_PATH = 'test/demo-v2/demo.idl';

beforeAll(async () => {
  api = await GearApi.create({ providerAddress: 'ws://127.0.0.1:9944' });
  await waitReady();
  const keyring = new Keyring({ type: 'sr25519' });
  alice = keyring.addFromUri('//Alice');
  charlie = keyring.addFromUri('//Charlie');
  charlieRaw = decodeAddress(charlie.address);
  code = readFileSync(DEMO_WASM_PATH);
  codeId = generateCodeHash(code);

  // Initialize Sails parser
  const parser = new SailsIdlParser();
  await parser.init();

  // Parse IDL
  const idlString = readFileSync(DEMO_IDL_PATH, 'utf8');
  const doc = parser.parse(idlString)

  // Create Sails Program from parsed IDL doc
  program = new SailsProgram(doc);
  program.setApi(api);
});

afterAll(async () => {
  await api.disconnect();
  await new Promise((resolve) => {
    setTimeout(resolve, 2000);
  });
});

describe('Sails with Parser v2 - Program Creation', () => {
  test('create program from code using Default constructor', async () => {
    const transaction = program.ctors.Default.fromCode(code).withAccount(alice).withGas('max');

    const { msgId, blockHash, response } = await transaction.signAndSend();

    expect(msgId).toBeDefined();
    expect(blockHash).toBeDefined();
    expect(transaction.programId).toBeDefined();

    await response();

    expect(program.programId).toBe(transaction.programId);
  });

  test('create program from code id using New constructor with parameters', async () => {
    const transaction = await program.ctors.New.fromCodeId(codeId, 10, [5, 8]).withAccount(alice).calculateGas();

    const { msgId, blockHash, response } = await transaction.signAndSend();

    expect(transaction.gasInfo).toBeDefined();
    expect(msgId).toBeDefined();
    expect(blockHash).toBeDefined();
    expect(transaction.programId).toBeDefined();

    // Update program ID for further tests
    programId = transaction.programId;
    program.setProgramId(programId);

    await response();
    expect(program.programId).toBe(programId);
  });
});

describe('Sails with Parser v2 - Message Sending', () => {
  test('send PingPong service ping message', async () => {
    const transaction = await program.services.PingPong.functions.Ping('ping').withAccount(alice).calculateGas();

    const { msgId, blockHash, response } = await transaction.signAndSend();

    expect(transaction.gasInfo).toBeDefined();
    expect(msgId).toBeDefined();
    expect(blockHash).toBeDefined();

    const result = await response();
    expect(result).toHaveProperty('ok', 'pong');
  });

  test('send Counter service add message', async () => {
    const transaction = await program.services.Counter.functions.Add(5).withAccount(alice).calculateGas();

    const { msgId, blockHash, response } = await transaction.signAndSend();

    expect(transaction.gasInfo).toBeDefined();
    expect(msgId).toBeDefined();
    expect(blockHash).toBeDefined();

    const result = await response();
    expect(result).toBe(15); // 10 (from constructor) + 5
  });

  test('send Counter service sub message', async () => {
    const transaction = await program.services.Counter.functions.Sub(3).withAccount(alice).calculateGas();

    const { msgId, blockHash, response } = await transaction.signAndSend();

    expect(transaction.gasInfo).toBeDefined();
    expect(msgId).toBeDefined();
    expect(blockHash).toBeDefined();

    const result = await response();
    expect(result).toBe(12); // 15 - 3
  });

  test('send Dog service makeSound message', async () => {
    const transaction = await program.services.Dog.functions.MakeSound().withAccount(alice).calculateGas();

    const { msgId, blockHash, response } = await transaction.signAndSend();

    expect(transaction.gasInfo).toBeDefined();
    expect(msgId).toBeDefined();
    expect(blockHash).toBeDefined();

    const result = await response();
    expect(result).toBe('Woof! Woof!');
  });

  test('send Dog service walk message', async () => {
    const transaction = await program.services.Dog.extends.WalkerService.functions.Walk(2, 3).withAccount(alice).calculateGas();

    const { msgId, blockHash, response } = await transaction.signAndSend();

    expect(transaction.gasInfo).toBeDefined();
    expect(msgId).toBeDefined();
    expect(blockHash).toBeDefined();

    const result = await response();
    expect(result).toBe(null);
  });

  test('send ThisThat service doThis message', async () => {
    const transaction = await program.services.ThisThat.functions
      .DoThis(1, 'test', [null, NonZeroU8(1)], [true])
      .withAccount(alice)
      .calculateGas();

    const { msgId, blockHash, response } = await transaction.signAndSend();

    expect(transaction.gasInfo).toBeDefined();
    expect(msgId).toBeDefined();
    expect(blockHash).toBeDefined();

    const result = await response();
    expect(result).toEqual(['test', 1]);
  });

  test('send ThisThat service doThat message', async () => {
    const transaction = await program.services.ThisThat.functions
      .DoThat({ p1: NonZeroU32(42), p2: ZERO_ADDRESS, p3: { Five: ['hello', H256(ZERO_ADDRESS)] } })
      .withAccount(alice)
      .calculateGas();

    const { msgId, blockHash, response } = await transaction.signAndSend();

    expect(transaction.gasInfo).toBeDefined();
    expect(msgId).toBeDefined();
    expect(blockHash).toBeDefined();

    const result = await response();
    expect(result).toEqual({ ok: [ZERO_ADDRESS, 42, 'Five'] });
  });
});

describe('Sails with Parser v2 - Queries', () => {
  test('query Counter value - basic call', async () => {
    const result = await program.services.Counter.queries.Value().call();
    expect(result).toBe(12);
  });

  test('query Counter value - with address', async () => {
    const result = await program.services.Counter.queries.Value().withAddress(alice.address).call();
    expect(result).toBe(12);
  });

  test('query Counter value - with full configuration', async () => {
    const result = await program.services.Counter.queries
      .Value()
      .withAddress(alice.address)
      .withValue(0n)
      .withGasLimit(1_000_000_000n)
      .call();
    expect(result).toBe(12);
  });

  test('query Dog position', async () => {
    const result = await program.services.Dog.extends.WalkerService.queries.Position().call();
    expect(result).toEqual([7, 11]); // [5, 8] from constructor + [2, 3] from walk
  });

  test('query Dog avgWeight', async () => {
    const result = await program.services.Dog.extends.MammalService.queries.AvgWeight().call();
    expect(typeof result).toBe('number');
  });

  test('query ThisThat this', async () => {
    const result = await program.services.ThisThat.queries.This().call();
    expect(typeof result).toBe('number');
  });

  test('query ThisThat that', async () => {
    const result = await program.services.ThisThat.queries.That().call();
    expect(result).toHaveProperty('ok');
  });

  test('query References baked', async () => {
    const result = await program.services.References.queries.Baked().call();
    expect(typeof result).toBe('string');
  });
});

describe('Sails with Parser v2 - Events', () => {
  test('subscribe to Counter Added event', async () => {
    let addedEventData: number | undefined;

    const unsubscribe = await program.services.Counter.events.Added.subscribe((data) => {
      addedEventData = data;
    });

    // Send a message that should trigger the event
    const transaction = await program.services.Counter.functions.Add(7).withAccount(alice).calculateGas();

    const { response } = await transaction.signAndSend();
    const result = await response();

    expect(result).toBe(19); // 12 + 7
    expect(addedEventData).toBe(7);

    unsubscribe();
  });

  test('subscribe to Counter Subtracted event', async () => {
    let subtractedEventData: number | undefined;

    const unsubscribe = await program.services.Counter.events.Subtracted.subscribe((data) => {
      subtractedEventData = data;
    });

    // Send a message that should trigger the event
    const transaction = await program.services.Counter.functions.Sub(4).withAccount(alice).calculateGas();

    const { response } = await transaction.signAndSend();
    const result = await response();

    expect(result).toBe(15); // 19 - 4
    expect(subtractedEventData).toBe(4);

    unsubscribe();
  });

  test('subscribe to Dog Barked event', async () => {
    let barkedEventTriggered = false;

    const unsubscribe = await program.services.Dog.events.Barked.subscribe(() => {
      barkedEventTriggered = true;
    });

    // Send a message that should trigger the event
    const transaction = await program.services.Dog.functions.MakeSound().withAccount(alice).calculateGas();

    const { response } = await transaction.signAndSend();
    const result = await response();

    expect(result).toBe('Woof! Woof!');
    expect(barkedEventTriggered).toBe(true);

    unsubscribe();
  });

  test('subscribe to Dog Walked event', async () => {
    let walkedEventData: { from: [number, number]; to: [number, number] } | undefined;

    const unsubscribe = await program.services.Dog.extends.WalkerService.events.Walked.subscribe((data) => {
      walkedEventData = data;
    });

    // Send a message that should trigger the event
    const transaction = await program.services.Dog.extends.WalkerService.functions.Walk(-1, 2).withAccount(alice).calculateGas();

    const { response } = await transaction.signAndSend();
    const result = await response();

    expect(result).toBe(null);
    expect(walkedEventData?.from).toEqual([7, 11]);
    expect(walkedEventData?.to).toEqual([6, 13]);

    unsubscribe();
  });
});

describe('Sails with Parser v2 - Payload Encoding/Decoding', () => {
  test('encode and decode constructor payload', async () => {
    const payload = program.ctors.New.encodePayload(25, [10, 20]);
    expect(payload).toBeDefined();
    expect(typeof payload).toBe('string');

    const decoded = program.ctors.New.decodePayload(payload);
    expect(decoded).toHaveProperty('counter', 25);
    expect(decoded).toHaveProperty('dog_position', [10, 20]);
  });

  test('encode and decode function payload', async () => {
    const payload = program.services.Counter.functions.Add.encodePayload(42);
    expect(payload).toBeDefined();
    expect(typeof payload).toBe('string');

    const decoded = program.services.Counter.functions.Add.decodePayload(payload);
    expect(decoded).toHaveProperty('value', 42);
  });

  test('encode and decode complex function payload', async () => {
    const param = { p1: NonZeroU32(99), p2: ZERO_ADDRESS, p3: { One: null } };
    const payload = program.services.ThisThat.functions.DoThat.encodePayload(param);
    expect(payload).toBeDefined();
    expect(typeof payload).toBe('string');

    const decoded = program.services.ThisThat.functions.DoThat.decodePayload(payload);
    expect(decoded).toHaveProperty('param');
    expect(decoded.param.p1).toBe(99);
    expect(decoded.param.p2).toBe(ZERO_ADDRESS);
  });
});

describe('Sails with Parser v2 - Error Handling', () => {
  test('transaction with insufficient gas should fail', async () => {
    const transaction = program.services.Counter.functions.Add(1).withAccount(alice).withGas(1000n); // Very low gas

    const { response } = await transaction.signAndSend();

    await expect(response()).rejects.toThrow('Message ran out of gas while executing.');
  });

  test('invalid query parameters should throw', async () => {
    expect(() => {
      program.services.Counter.queries.Value().withAddress('invalid-address');
    }).toThrow('Invalid address.');
  });
});

describe('Sails with Parser v2 - Voucher Support', () => {
  test('send message with voucher', async () => {
    // Create a voucher first
    const { extrinsic, voucherId } = await api.voucher.issue(charlieRaw, 10 * 1e12);

    await new Promise((resolve, reject) =>
      extrinsic.signAndSend(alice, ({ events, status }) => {
        if (status.isInBlock) {
          const success = events.find((record) => record.event.method === 'ExtrinsicSuccess');
          if (success) {
            resolve(voucherId);
          } else {
            reject(new Error('Extrinsic failed'));
          }
        }
      }),
    );

    // Use the voucher for a transaction
    const transaction = await program.services.Counter.functions
      .Add(3)
      .withAccount(charlie)
      .withVoucher(voucherId)
      .calculateGas();

    const { response } = await transaction.signAndSend();
    const result = await response();

    expect(transaction.gasInfo).toBeDefined();
    expect(result).toBe(18); // 15 + 3
  });
});

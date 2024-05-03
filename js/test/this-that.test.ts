import { readFileSync } from 'fs';
import { hexToU8a } from '@polkadot/util';
import { Sails } from '../lib';

let sails: Sails;
const IDL_PATH = '../examples/this-that-svc/wasm/this-that-svc.idl';

beforeAll(async () => {
  sails = await Sails.new();
});

describe('this-that', () => {
  test('parse idl', () => {
    const idl = readFileSync(IDL_PATH, 'utf-8');

    sails.parseIdl(idl);

    expect(sails.services).toHaveProperty('Service');

    expect(sails.scaleCodecTypes).toHaveProperty('DoThatParam');
    expect(sails.scaleCodecTypes).toHaveProperty('ManyVariants');
    expect(sails.scaleCodecTypes).toHaveProperty('TupleStruct');
    expect(sails.scaleCodecTypes.TupleStruct).toEqual('(bool)');
    expect(sails.scaleCodecTypes.DoThatParam).toEqual({
      p1: 'u32',
      p2: 'String',
      p3: 'ManyVariants',
    });
    expect(sails.scaleCodecTypes.ManyVariants).toEqual({
      _enum: {
        One: 'Null',
        Two: 'u32',
        Three: 'Option<U256>',
        Four: { a: 'u32', b: 'Option<u16>' },
        Five: '(String, H256)',
        Six: '(u32)',
      },
    });

    expect(sails.services.Service.functions).toHaveProperty('DoThis');
    expect(sails.services.Service.functions).toHaveProperty('DoThat');
    expect(sails.services.Service.functions).toHaveProperty('That');
    expect(sails.services.Service.functions).toHaveProperty('This');
  });

  test('encode/decode', async () => {
    const h256Hash = '0x' + Buffer.from(new Array(32).fill(0)).toString('hex');

    const payloadWithH256 = sails.services.Service.functions.DoThat.encodePayload({
      p1: 1,
      p2: 'hello',
      p3: { five: ['str', h256Hash] },
    });

    expect(Array.from(hexToU8a(payloadWithH256))).toEqual([
      28, 83, 101, 114, 118, 105, 99, 101, 24, 68, 111, 84, 104, 97, 116, 1, 0, 0, 0, 20, 104, 101, 108, 108, 111, 4,
      12, 115, 116, 114, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ]);

    const payloadWithU256 = sails.services.Service.functions.DoThat.encodePayload({
      p1: 2,
      p2: 'world',
      p3: { three: 1234567890 },
    });

    expect(Array.from(hexToU8a(payloadWithU256))).toEqual([
      28, 83, 101, 114, 118, 105, 99, 101, 24, 68, 111, 84, 104, 97, 116, 2, 0, 0, 0, 20, 119, 111, 114, 108, 100, 2, 1,
      210, 2, 150, 73, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ]);

    const decodedWithH256 = sails.services.Service.functions.DoThat.decodePayload(payloadWithH256);

    expect(decodedWithH256).toEqual({
      p1: 1,
      p2: 'hello',
      p3: { five: ['str', h256Hash] },
    });

    const decodedWithU256 = sails.services.Service.functions.DoThat.decodePayload(payloadWithU256);
    decodedWithU256.p3.three = BigInt(decodedWithU256.p3.three); // TODO: find a better way to handle this

    expect(decodedWithU256).toEqual({
      p1: 2,
      p2: 'world',
      p3: { three: 1234567890n },
    });
  });
});

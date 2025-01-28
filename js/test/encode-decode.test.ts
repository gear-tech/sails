import { readFileSync } from 'node:fs';
import { getCtorNamePrefix, getFnNamePrefix as getFunctionNamePrefix, getServiceNamePrefix, Sails } from '..';
import { SailsIdlParser } from 'sails-js-parser';

const DEMO_IDL_PATH = '../examples/demo/client/demo.idl';

let sails: Sails;
const demoIdl = readFileSync(DEMO_IDL_PATH, 'utf8');

beforeAll(async () => {
  const parser = await SailsIdlParser.new();
  sails = new Sails(parser);
  sails.parseIdl(demoIdl);
});

describe('Encode/Decode', () => {
  test('encode/decode demo ctors', async () => {
    const newEncoded = sails.ctors.New.encodePayload(null, [5, 5]);

    expect(newEncoded).toBe('0x0c4e657700010500000005000000');

    const newDecoded = sails.ctors.New.decodePayload(newEncoded);

    expect(newDecoded).toEqual({ counter: null, dog_position: [5, 5] });

    expect(getCtorNamePrefix(newEncoded)).toBe('New');
  });

  test('encode/decode dog walk', async () => {
    const walkEncoded = sails.services.Dog.functions.Walk.encodePayload(10, 10);

    expect(walkEncoded).toBe('0x0c446f671057616c6b0a0000000a000000');

    const walkDecoded = sails.services.Dog.functions.Walk.decodePayload(walkEncoded);

    expect(walkDecoded).toEqual({ dx: 10, dy: 10 });

    expect(getServiceNamePrefix(walkEncoded)).toBe('Dog');
    expect(getFunctionNamePrefix(walkEncoded)).toBe('Walk');
  });
});

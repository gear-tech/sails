import { performance } from 'node:perf_hooks';

import { SailsIdlParser, InterfaceId, SailsMessageHeader } from 'sails-js-parser-idl-v2';

import { SailsProgram } from '../src/sails-idl-v2';

const idl = `
  service Counter {
    events {
      @entry_id: 1
      Added(u32),
    }
  }

  program CounterProgram {
    constructors {
      New();
    }
    services {
      Counter
    }
  }
`;

const concat = (left: Uint8Array, right: Uint8Array): any => {
  const out = new Uint8Array(left.length + right.length);
  out.set(left, 0);
  out.set(right, left.length);
  return out;
};

describe('decode dispatcher benchmark', () => {
  test('decodeEvent p99 stays under 200us for a reused SailsProgram', async () => {
    const parser = new SailsIdlParser();
    await parser.init();
    const program = new SailsProgram(parser.parse(idl));
    const iid = InterfaceId.from((program as any)._doc.services[0].interface_id);
    const header = SailsMessageHeader.v1(iid, 1, 1);
    const body = program.registry.createType<any>('u32', 7).toU8a();
    const bytes = concat(header.toBytes(), body);

    for (let i = 0; i < 100; i += 1) {
      (program as any).decodeEvent(bytes);
    }

    const timings: number[] = [];
    for (let i = 0; i < 10_000; i += 1) {
      const start = performance.now();
      const decoded = (program as any).decodeEvent(bytes);
      const end = performance.now();
      if (decoded.kind !== 'event') {
        throw new Error(`unexpected decode result: ${JSON.stringify(decoded)}`);
      }
      timings.push((end - start) * 1000);
    }

    timings.sort((a, b) => a - b);
    const p99 = timings[Math.floor(timings.length * 0.99)];
    expect(p99).toBeLessThan(200);
  });
});

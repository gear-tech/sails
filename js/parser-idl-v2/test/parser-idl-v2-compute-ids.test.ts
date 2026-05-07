import { SailsIdlParser } from '..';

describe('parser-v2 computeInterfaceIds', () => {
  test('throws when not initialized', () => {
    const parser = new SailsIdlParser();
    expect(() => parser.computeInterfaceIds('service S { functions { Ping() -> bool; } }')).toThrow(
      'SailsIdlParser is not initialized. Call init() first.',
    );
  });

  test('returns ids as a plain Record<name, hex>', async () => {
    const parser = new SailsIdlParser();
    await parser.init();

    const ids = parser.computeInterfaceIds(`
      service Counter {
        functions { Add(value: u32) -> u32; }
      }
    `);

    expect(Object.keys(ids)).toEqual(['Counter']);
    expect(ids.Counter).toMatch(/^0x[0-9a-f]{16}$/);
  });

  test('ignores placeholder mismatches that parse() rejects', async () => {
    const parser = new SailsIdlParser();
    await parser.init();

    const idl = `
      service Counter@0x0000000000000000 {
        functions { Add(value: u32) -> u32; }
      }
    `;

    expect(() => parser.parse(idl)).toThrow(/computed interface_id .* is not equal to/);

    const ids = parser.computeInterfaceIds(idl);
    expect(ids.Counter).toMatch(/^0x[0-9a-f]{16}$/);
    expect(ids.Counter).not.toBe('0x0000000000000000');
  });

  test('matches ids returned by parse() for the same source', async () => {
    const parser = new SailsIdlParser();
    await parser.init();

    const idl = `
      service Base {
        functions { Ping() -> bool; }
      }
      service Derived {
        extends { Base }
        functions { Pong() -> bool; }
      }
    `;

    const ids = parser.computeInterfaceIds(idl);
    const doc = parser.parse(idl);

    for (const service of doc.services ?? []) {
      const fromCompute = ids[service.name];
      const fromParse = service.interface_id?.toString();
      expect(fromCompute).toBeDefined();
      expect(fromParse).toBeDefined();
      expect(fromCompute).toBe(fromParse);
    }
  });

  test('passes through @partial services with explicit ids', async () => {
    const parser = new SailsIdlParser();
    await parser.init();

    const ids = parser.computeInterfaceIds(`
      @partial
      service Ext@0x0123456789abcdef {
        functions {
          @entry_id: 0
          Ping() -> bool;
        }
      }
    `);

    expect(ids.Ext).toBe('0x0123456789abcdef');
  });

  test('errors on @partial without explicit id', async () => {
    const parser = new SailsIdlParser();
    await parser.init();

    expect(() =>
      parser.computeInterfaceIds(`
        @partial
        service Ext {
          functions {
            @entry_id: 0
            Ping() -> bool;
          }
        }
      `),
    ).toThrow(/@partial/);
  });

  test('returns empty map for whitespace-only IDL', async () => {
    const parser = new SailsIdlParser();
    await parser.init();

    // The WASM FFI rejects truly empty input (matches parse_idl_to_json behavior),
    // so use whitespace as a no-services document.
    const ids = parser.computeInterfaceIds('   \n  ');
    expect(ids).toEqual({});
  });

  test('propagates parse errors', async () => {
    const parser = new SailsIdlParser();
    await parser.init();

    expect(() => parser.computeInterfaceIds('this is not valid idl')).toThrow(/Error code:/);
  });
});

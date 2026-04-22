import { SailsIdlParser } from '..';

describe('parser-v2 success', () => {
  test('parses demo.idl', async () => {
    const parser = new SailsIdlParser();
    await parser.init();

    const idl = await import('./fixture/demo.js')
    const doc = parser.parse(idl.default);

    expect(doc.program?.name).toBe('DemoClient');
    expect(doc.services?.map((service) => service.name)).toEqual([
      'PingPong',
      'Counter',
      'MammalService',
      'WalkerService',
      'Dog',
      'References',
      'ThisThat',
      'ValueFee',
      'Validator',
      'Chaos',
      'Chain',
      "BaseService",
      "OverrideGenerics"
    ]);
  });

  test('parses idl with aliases', async () => {
    const parser = new SailsIdlParser();
    await parser.init();

    const idl = `
      service S {
        types {
          struct T { f: u32 }
          alias A = T;
        }
      }
    `;
    const doc = parser.parse(idl);

    const types = doc.services?.[0].types;
    expect(types?.[1].name).toBe('A');
    expect(types?.[1].kind).toBe('alias');
    // @ts-ignore
    expect(types?.[1].target).toEqual({ kind: 'named', name: 'T' });
  });

  test('normalizes generic type parameters', async () => {
    const parser = new SailsIdlParser();
    await parser.init();

    const idl = `
      service S {
        types {
          struct Wrapper<T>(T, Option<T>);
        }
      }
    `;
    const doc = parser.parse(idl);

    const wrapper = doc.services?.[0].types?.[0];
    expect(wrapper?.name).toBe('Wrapper');
    expect(wrapper?.kind).toBe('struct');
    // @ts-ignore
    expect(wrapper?.fields[0].type).toEqual({ kind: 'generic', name: 'T' });
    // @ts-ignore
    expect(wrapper?.fields[1].type).toEqual({
      kind: 'named',
      name: 'Option',
      generics: [{ kind: 'generic', name: 'T' }],
    });
  });
});

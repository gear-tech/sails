import { SailsIdlParser } from 'sails-js-parser-idl-v2';

import { SailsProgram, type ITypeStruct, type Type } from '..';

let parser: SailsIdlParser;

beforeAll(async () => {
  parser = new SailsIdlParser();
  await parser.init();
});

describe('SailsProgram.programTypes', () => {
  test('returns a map of program-level user types keyed by name', () => {
    const text = `
      program Test {
        types {
          struct Shared {
            v: u32,
          }
          enum Mood {
            Happy,
            Sad,
          }
        }
        constructors {
          Default(p: Shared);
        }
      }
    `;
    const program = new SailsProgram(parser.parse(text));

    expect(program.programTypes.size).toBe(2);

    const shared = program.programTypes.get('Shared');
    expect(shared?.kind).toBe('struct');
    expect((shared as ITypeStruct).fields).toHaveLength(1);

    const mood = program.programTypes.get('Mood');
    expect(mood?.kind).toBe('enum');
  });

  test('returns an empty map when the program block has no types', () => {
    const text = `
      program Test {
        constructors {
          Default();
        }
      }
    `;
    const program = new SailsProgram(parser.parse(text));

    expect(program.programTypes.size).toBe(0);
  });

  test('returns an empty map when the IDL has no program block', () => {
    // Services-only IDL: legal per the v2 spec for service interface bundles.
    const text = `
      !@sails: 1.0.0-beta.3

      service Foo {
        functions {
          Ping() -> u32;
        }
      }
    `;
    const program = new SailsProgram(parser.parse(text));

    expect(program.programTypes.size).toBe(0);
  });

  test('returns the same Map instance on repeated access (eager construction)', () => {
    const text = `
      program Test {
        types {
          struct A { v: u32 }
        }
        constructors {
          Default();
        }
      }
    `;
    const program = new SailsProgram(parser.parse(text));

    expect(program.programTypes).toBe(program.programTypes);
  });
});

describe('SailsService.types', () => {
  test('returns a map of types declared in the service block', () => {
    const text = `
      !@sails: 1.0.0-beta.3

      service Foo {
        functions {
          Ping() -> Local;
        }
        types {
          struct Local {
            v: u32,
          }
        }
      }

      program Test {
        constructors {
          Default();
        }
        services {
          Foo,
        }
      }
    `;
    const program = new SailsProgram(parser.parse(text));
    const foo = program.services.Foo;

    expect(foo.types.size).toBe(1);
    expect(foo.types.get('Local')?.kind).toBe('struct');
  });

  test('returns an empty map when the service has no types block', () => {
    const text = `
      !@sails: 1.0.0-beta.3

      service Foo {
        functions {
          Ping() -> u32;
        }
      }

      program Test {
        constructors {
          Default();
        }
        services {
          Foo,
        }
      }
    `;
    const program = new SailsProgram(parser.parse(text));

    expect(program.services.Foo.types.size).toBe(0);
  });

  test('declared-only: types declared in service A are not in service B.types', () => {
    const text = `
      !@sails: 1.0.0-beta.3

      service Alpha {
        functions {
          Ping() -> InAlpha;
        }
        types {
          struct InAlpha {
            v: u32,
          }
        }
      }

      service Beta {
        functions {
          Ping() -> InBeta;
        }
        types {
          struct InBeta {
            w: u32,
          }
        }
      }

      program Test {
        constructors {
          Default();
        }
        services {
          Alpha,
          Beta,
        }
      }
    `;
    const program = new SailsProgram(parser.parse(text));

    expect(program.services.Alpha.types.has('InAlpha')).toBe(true);
    expect(program.services.Alpha.types.has('InBeta')).toBe(false);

    expect(program.services.Beta.types.has('InBeta')).toBe(true);
    expect(program.services.Beta.types.has('InAlpha')).toBe(false);
  });

  test('declared-only: base types are not pulled into the derived service via extends', () => {
    // `Child` extends `Base`. `Base` defines `Shared`. The declared-only contract
    // says `Child.types` must NOT include `Shared` — that's only reachable via
    // `Child.extends.Base.types` or `program.resolveInService('Child', ...)`.
    const text = `
      !@sails: 1.0.0-beta.3

      service Base {
        functions {
          Ping() -> u32;
        }
        types {
          struct Shared {
            v: u32,
          }
        }
      }

      service Child {
        extends {
          Base,
        }
      }

      program Test {
        constructors {
          Default();
        }
        services {
          Child,
        }
      }
    `;
    const program = new SailsProgram(parser.parse(text));
    const child = program.services.Child;

    expect(child.types.size).toBe(0);
    expect(child.types.has('Shared')).toBe(false);

    // Sanity check: the base type is reachable via the extends accessor.
    expect(child.extends.Base.types.has('Shared')).toBe(true);

    // Sanity check: resolveInService still merges the extends chain.
    const resolved = program.resolveInService('Child', { kind: 'named', name: 'Shared' });
    expect(resolved?.kind).toBe('struct');
  });

  test('returns the same Map instance on repeated access (eager construction)', () => {
    const text = `
      !@sails: 1.0.0-beta.3

      service Foo {
        functions { Ping() -> u32; }
        types {
          struct Local { v: u32 }
        }
      }

      program Test {
        constructors {
          Default();
        }
        services {
          Foo,
        }
      }
    `;
    const program = new SailsProgram(parser.parse(text));
    const foo = program.services.Foo;

    expect(foo.types).toBe(foo.types);
  });
});

describe('top-level type re-export from sails-js', () => {
  test('AST types `Type` and `ITypeStruct` import from the package root', () => {
    // Type-level proof: this file imports `Type` and `ITypeStruct` from `'..'`
    // (the sails-js root entry, not the `./types` subpath). If the re-export
    // were missing, the imports above and the assignment below would not
    // compile — jest's TS pass exercises the check at test build time.
    const text = `
      program Test {
        types {
          struct Shared { v: u32 }
        }
        constructors {
          Default();
        }
      }
    `;
    const program = new SailsProgram(parser.parse(text));
    const t: Type | undefined = program.programTypes.get('Shared');
    const s: ITypeStruct | undefined = t?.kind === 'struct' ? (t as ITypeStruct) : undefined;

    expect(s?.fields).toHaveLength(1);
  });
});

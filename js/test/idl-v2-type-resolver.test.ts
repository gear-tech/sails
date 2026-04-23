import { TypeRegistry } from '@polkadot/types/create';
import type { Type, TypeDecl } from 'sails-js-types';

import { TypeResolver } from '../src/type-resolver-idl-v2.js';

const named = (name: string, generics?: TypeDecl[]): TypeDecl => ({
  kind: 'named',
  name,
  generics,
});

const generic = (name: string): TypeDecl => ({
  kind: 'generic',
  name,
});

describe('type-resolver-v2 generics', () => {
  test('resolves explicit generic leaves', () => {
    const resolver = new TypeResolver([]);

    expect(resolver.getTypeDeclString(generic('T'), { T: 'u32' })).toBe('u32');
    expect(resolver.getTypeDeclString(generic('T'))).toBe('T');
  });

  test('registers generic types', () => {
    const registry = new TypeRegistry();
    registry.register({
      "SimpleStruct": {
        a: 'String',
        b: 'u32',
      },
      "Wrapper<u32>": {
        value: 'u32',
        items: 'Vec<u32>',
      },
      "Pair<u8,String>": {
        left: "u8",
        right: "Option<String>"
      }
    })

    const encoded = registry.createType('SimpleStruct', { a: 'hello', b: 123 });
    expect(encoded.toJSON()).toEqual({
      a: 'hello',
      b: 123,
    });

    const encoded2 = registry.createType("Wrapper<u32>", { value: 42, items: [0, 1, 2] });
    expect(encoded2.toJSON()).toEqual({ value: 42, items: [0, 1, 2] });

    const encoded3 = registry.createType("Pair<u8,String>", { left: 42, right: 'hello' });
    expect(encoded3.toJSON()).toEqual({ left: 42, right: 'hello' });

    const encoded4 = registry.createType("Pair<u8,String>", { left: 42, right: null });
    expect(encoded4.toJSON()).toEqual({ left: 42, right: null });
  });

  test('registers generic structs with resolved fields', () => {
    const userTypes: Type[] = [
      {
        kind: 'struct',
        name: 'Wrapper',
        type_params: [{ name: 'T' }],
        fields: [
          { name: 'value', type: generic('T') },
          { name: 'items', type: { kind: 'slice', item: generic('T') } },
        ],
      },
      {
        kind: 'struct',
        name: 'Pair',
        type_params: [{ name: 'T' }, { name: 'U' }],
        fields: [
          { name: 'left', type: generic('T') },
          { name: 'right', type: named('Option', [generic('U')]) },
        ],
      },
    ];

    const resolver = new TypeResolver(userTypes);

    expect(resolver.getTypeDef(userTypes[0], { "T": "u32" })).toEqual({
      value: 'u32',
      items: 'Vec<u32>',
    });
    const wrapperDecl = resolver.getTypeDeclString(
      { kind: 'named', name: 'Wrapper', generics: ['u32'] },
    );
    expect(wrapperDecl).toBe('Wrapper<u32>');
    expect(resolver.registry.hasType('Wrapper<u32>')).toBe(true);

    const encoded = resolver.registry.createType("Wrapper<u32>", { value: 42, items: [0, 1, 2] });
    expect(encoded.toJSON()).toEqual({ value: 42, items: [0, 1, 2] });

    const pairDecl = resolver.getTypeDeclString(
      { kind: 'named', name: 'Pair', generics: ['u8', 'String'] },
    );
    expect(pairDecl).toBe('Pair<u8,String>');
    expect(resolver.registry.hasType('Pair<u8,String>')).toBe(true);

    const encoded3 = resolver.registry.createType('Pair<u8,String>', { left: 42, right: 'hello' });
    expect(encoded3.toJSON()).toEqual({ left: 42, right: 'hello' });

    const encoded4 = resolver.registry.createType("Pair<u8,String>", { left: 42, right: null });
    expect(encoded4.toJSON()).toEqual({ left: 42, right: null });
  });

  test('registers generic enums with resolved variants', () => {
    const userTypes: Type[] = [
      {
        kind: 'enum',
        name: 'Maybe',
        type_params: [{ name: 'T' }],
        variants: [
          { name: 'None', fields: [] },
          { name: 'Some', fields: [{ type: generic('T') }] },
        ],
      },
    ];

    const resolver = new TypeResolver(userTypes);
    const maybeDecl = resolver.getTypeDeclString(
      { kind: 'named', name: 'Maybe', generics: ['String'] },
    );
    expect(maybeDecl).toBe('Maybe<String>');
    expect(resolver.registry.hasType(maybeDecl)).toBe(true);

    const encoded = resolver.registry.createType('Maybe<String>', { Some: 'hello' });
    expect(encoded.toJSON()).toEqual({ some: 'hello' });
  });
});

describe('type-resolver-v2 structs', () => {
  test('simple struct', () => {
    const userType: Type = {
      kind: 'struct',
      name: 'SimpleStruct',
      fields: [
        { name: 'a', type: 'String' },
        { name: 'b', type: 'u32' },
      ],
    };

    const resolver = new TypeResolver([userType]);

    expect(resolver.getTypeDef(userType)).toEqual({
      a: 'String',
      b: 'u32',
    });

    const encoded = resolver.registry.createType('SimpleStruct', { a: 'hello', b: 123 });
    expect(encoded.toJSON()).toEqual({
      a: 'hello',
      b: 123,
    });
  });

  test('struct with option', () => {
    const userType: Type = {
      kind: 'struct',
      name: 'StructWithOption',
      fields: [
        { name: 'a', type: named('Option', ['String']) },
        { name: 'b', type: 'u32' },
      ],
    };

    const resolver = new TypeResolver([userType]);

    expect(resolver.getTypeDef(userType)).toEqual({
      a: 'Option<String>',
      b: 'u32',
    });

    let encoded = resolver.registry.createType('StructWithOption', { a: 'hello', b: 123 });
    expect(encoded.toJSON()).toEqual({
      a: 'hello',
      b: 123,
    });

    encoded = resolver.registry.createType('StructWithOption', { a: null, b: 123 });
    expect(encoded.toJSON()).toEqual({
      a: null,
      b: 123,
    });
  });

  test('struct with result', () => {
    const userType: Type = {
      kind: 'struct',
      name: 'StructWithResult',
      fields: [
        { name: 'a', type: named('Result', ['String', 'u32']) },
        { name: 'b', type: 'u32' },
      ],
    };

    const resolver = new TypeResolver([userType]);

    expect(resolver.getTypeDef(userType)).toEqual({
      a: 'Result<String,u32>',
      b: 'u32',
    });

    let encoded = resolver.registry.createType('StructWithResult', { a: { ok: 'hello' }, b: 123 });
    expect(encoded.toJSON()).toEqual({
      a: { ok: 'hello' },
      b: 123,
    });

    encoded = resolver.registry.createType('StructWithResult', { a: { err: 123 }, b: 123 });
    expect(encoded.toJSON()).toEqual({
      a: { err: 123 },
      b: 123,
    });
  });

  test('struct with tuple', () => {
    const userType: Type = {
      kind: 'struct',
      name: 'StructWithTuple',
      fields: [
        { name: 'a', type: { kind: 'tuple', types: ['String', 'u32'] } },
        { name: 'b', type: 'u32' },
      ],
    };

    const resolver = new TypeResolver([userType]);

    expect(resolver.getTypeDef(userType)).toEqual({
      a: '(String,u32)',
      b: 'u32',
    });

    const encoded = resolver.registry.createType('StructWithTuple', { a: ['hello', 123], b: 123 });
    expect(encoded.toJSON()).toEqual({
      a: ['hello', 123],
      b: 123,
    });
  });

  test('struct with vec', () => {
    const userType: Type = {
      kind: 'struct',
      name: 'StructWithVec',
      fields: [
        { name: 'a', type: { kind: 'slice', item: 'String' } },
        { name: 'b', type: 'u32' },
      ],
    };

    const resolver = new TypeResolver([userType]);

    expect(resolver.getTypeDef(userType)).toEqual({
      a: 'Vec<String>',
      b: 'u32',
    });

    const encoded = resolver.registry.createType('StructWithVec', { a: ['hello', 'world'], b: 123 });
    expect(encoded.toJSON()).toEqual({
      a: ['hello', 'world'],
      b: 123,
    });
  });

  test('struct with fixed size array', () => {
    const userType: Type = {
      kind: 'struct',
      name: 'StructWithArray',
      fields: [
        { name: 'a', type: { kind: 'array', item: 'u32', len: 3 } },
        { name: 'b', type: 'u32' },
      ],
    };

    const resolver = new TypeResolver([userType]);

    expect(resolver.getTypeDef(userType)).toEqual({
      a: '[u32;3]',
      b: 'u32',
    });

    const encoded = resolver.registry.createType('StructWithArray', { a: [1, 2, 3], b: 123 });
    expect(encoded.toJSON()).toEqual({
      a: [1, 2, 3],
      b: 123,
    });
  });
});

describe('type-resolver-v2 substituteGenerics', () => {
  const resolver = new TypeResolver([]);

  test('replaces a bare named type_param with a primitive', () => {
    expect(resolver.substituteGenerics(named('T'), { T: 'u32' })).toBe('u32');
  });

  test('recurses through slice / array / tuple', () => {
    const input: TypeDecl = {
      kind: 'tuple',
      types: [
        { kind: 'slice', item: named('T') },
        { kind: 'array', item: named('T'), len: 4 },
      ],
    };
    expect(resolver.substituteGenerics(input, { T: 'u8' })).toEqual({
      kind: 'tuple',
      types: [
        { kind: 'slice', item: 'u8' },
        { kind: 'array', item: 'u8', len: 4 },
      ],
    });
  });

  test('recurses through named-with-generics (Option<T>, custom wrappers)', () => {
    const input: TypeDecl = named('Envelope', [named('Option', [named('T')])]);
    expect(
      resolver.substituteGenerics(input, { T: { kind: 'slice', item: 'u8' } }),
    ).toEqual(named('Envelope', [named('Option', [{ kind: 'slice', item: 'u8' }])]));
  });

  test('passes through unknown named refs that are not in substitutions', () => {
    expect(resolver.substituteGenerics(named('Unknown'), {})).toEqual(named('Unknown'));
  });

  test('is a no-op on primitives and on inputs with no type_params', () => {
    expect(resolver.substituteGenerics('u32')).toBe('u32');
    expect(resolver.substituteGenerics({ kind: 'slice', item: 'u8' }, { T: 'u64' })).toEqual({
      kind: 'slice',
      item: 'u8',
    });
  });

  test('is idempotent', () => {
    const input: TypeDecl = named('Envelope', [named('T')]);
    const once = resolver.substituteGenerics(input, { T: 'u32' });
    expect(resolver.substituteGenerics(once, { T: 'u32' })).toEqual(once);
  });

  test('does not mutate the input tree', () => {
    const input: TypeDecl = { kind: 'slice', item: named('T') };
    const snapshot = JSON.parse(JSON.stringify(input));
    resolver.substituteGenerics(input, { T: 'u8' });
    expect(input).toEqual(snapshot);
  });

  test('resolves substitution chains (T -> U -> u32)', () => {
    expect(
      resolver.substituteGenerics(named('T'), { T: named('U'), U: 'u32' }),
    ).toBe('u32');
  });

  test('throws on self-referential substitution map', () => {
    expect(() => resolver.substituteGenerics(named('T'), { T: named('T') })).toThrow(/[Cc]yclic/);
  });

  test('throws on cyclic substitution chain (T -> U -> T)', () => {
    expect(() =>
      resolver.substituteGenerics(named('T'), { T: named('U'), U: named('T') }),
    ).toThrow(/[Cc]yclic/);
  });

  test('throws on unknown TypeDecl kind', () => {
    // A `Type` (kind: 'struct') is not a valid TypeDecl — catch the misuse loudly.
    const bogus = { kind: 'struct', name: 'X', fields: [] } as unknown as TypeDecl;
    expect(() => resolver.substituteGenerics(bogus)).toThrow(/Unknown TypeDecl kind/);
  });
});

describe('type-resolver-v2 resolveNamed', () => {
  const packet: Type = {
    kind: 'struct',
    name: 'Packet',
    fields: [{ name: 'payload', type: { kind: 'array', item: 'u8', len: 4 } }],
  };
  const envelope: Type = {
    kind: 'struct',
    name: 'Envelope',
    type_params: [{ name: 'T' }],
    fields: [
      { name: 'id', type: 'u32' },
      { name: 'payload', type: named('T') },
    ],
  };

  test('returns the user Type for a known named decl', () => {
    const resolver = new TypeResolver([packet]);
    expect(resolver.resolveNamed(named('Packet'))).toBe(packet);
  });

  test('returns the user Type for a generic named decl (ignoring generics)', () => {
    const resolver = new TypeResolver([envelope]);
    expect(resolver.resolveNamed(named('Envelope', ['u32']))).toBe(envelope);
  });

  test('returns undefined for primitives, slices, arrays, tuples', () => {
    const resolver = new TypeResolver([]);
    expect(resolver.resolveNamed('u32')).toBeUndefined();
    expect(resolver.resolveNamed({ kind: 'slice', item: 'u8' })).toBeUndefined();
    expect(resolver.resolveNamed({ kind: 'array', item: 'u8', len: 4 })).toBeUndefined();
    expect(resolver.resolveNamed({ kind: 'tuple', types: ['u8', 'u16'] })).toBeUndefined();
  });

  test('returns undefined for unknown names and bare type_params', () => {
    const resolver = new TypeResolver([]);
    expect(resolver.resolveNamed(named('Unknown'))).toBeUndefined();
    expect(resolver.resolveNamed(named('T'))).toBeUndefined();
  });
});

describe('type-resolver-v2 genericsSubstitutions', () => {
  test('zips type_params with concrete generics', () => {
    const resolver = new TypeResolver([]);
    const userType: Type = {
      kind: 'struct',
      name: 'Pair',
      type_params: [{ name: 'T' }, { name: 'U' }],
      fields: [
        { name: 'left', type: named('T') },
        { name: 'right', type: named('U') },
      ],
    };
    expect(resolver.genericsSubstitutions(userType, ['u32', 'String'])).toEqual({
      T: 'u32',
      U: 'String',
    });
  });

  test('returns empty map when userType has no type_params', () => {
    const resolver = new TypeResolver([]);
    const userType: Type = {
      kind: 'struct',
      name: 'Plain',
      fields: [{ name: 'x', type: 'u32' }],
    };
    expect(resolver.genericsSubstitutions(userType, ['u32'])).toEqual({});
  });

  test('ignores extra concrete generics past the declared params', () => {
    const resolver = new TypeResolver([]);
    const userType: Type = {
      kind: 'struct',
      name: 'One',
      type_params: [{ name: 'T' }],
      fields: [{ name: 'v', type: named('T') }],
    };
    expect(resolver.genericsSubstitutions(userType, ['u8', 'u16'])).toEqual({ T: 'u8' });
  });
});

describe('type-resolver-v2 ambient types', () => {
  const ambientPacket: Type = {
    kind: 'struct',
    name: 'Packet',
    fields: [{ name: 'payload', type: { kind: 'array', item: 'u8', len: 4 } }],
  };
  const localPacket: Type = {
    kind: 'struct',
    name: 'Packet',
    fields: [{ name: 'payload', type: { kind: 'array', item: 'u8', len: 8 } }],
  };
  const ambientOnly: Type = {
    kind: 'struct',
    name: 'AmbientOnly',
    fields: [{ name: 'v', type: 'u32' }],
  };

  test('ambient types are resolvable when not shadowed', () => {
    const resolver = new TypeResolver([], [ambientOnly]);
    expect(resolver.resolveNamed(named('AmbientOnly'))).toBe(ambientOnly);
  });

  test('service-local types shadow ambients on name collision', () => {
    const resolver = new TypeResolver([localPacket], [ambientPacket]);
    expect(resolver.resolveNamed(named('Packet'))).toBe(localPacket);
  });

  test('registry reflects shadowing (local wins)', () => {
    const resolver = new TypeResolver([localPacket], [ambientPacket]);
    // With localPacket (len 8), an 8-byte array round-trips; a 4-byte payload would fail.
    const encoded = resolver.registry.createType('Packet', { payload: [1, 2, 3, 4, 5, 6, 7, 8] });
    expect(encoded.toJSON()).toEqual({ payload: '0x0102030405060708' });
    // Attempting to create with only 4 bytes should fail (ambient shape was overridden).
    expect(() => resolver.registry.createType('Packet', { payload: [1, 2, 3, 4] })).toThrow();
  });
});

describe('type-resolver-v2 aliases', () => {
  test('simple alias', () => {
    const userType: any = {
      kind: 'alias',
      name: 'MyU32',
      target: 'u32',
    };

    const resolver = new TypeResolver([userType]);

    expect(resolver.getTypeDef(userType)).toBe('u32');

    const encoded = resolver.registry.createType('MyU32', 123);
    expect(encoded.toJSON()).toBe(123);
  });

  test('alias to struct', () => {
    const structType: Type = {
      kind: 'struct',
      name: 'SimpleStruct',
      fields: [{ name: 'a', type: 'u32' }],
    };
    const aliasType: any = {
      kind: 'alias',
      name: 'StructAlias',
      target: named('SimpleStruct'),
    };

    const resolver = new TypeResolver([structType, aliasType]);

    expect(resolver.getTypeDef(aliasType)).toBe('SimpleStruct');

    const encoded = resolver.registry.createType('StructAlias', { a: 123 });
    expect(encoded.toJSON()).toEqual({ a: 123 });
  });

  test('generic alias', () => {
    const aliasType: any = {
      kind: 'alias',
      name: 'GenericAlias',
      type_params: [{ name: 'T' }],
      target: named('Option', [generic('T')]),
    };

    const resolver = new TypeResolver([aliasType]);

    const decl = resolver.getTypeDeclString(named('GenericAlias', ['u32']));
    expect(decl).toBe('GenericAlias<u32>');

    const encoded = resolver.registry.createType('GenericAlias<u32>', 123);
    expect(encoded.toJSON()).toBe(123);

    const encodedNull = resolver.registry.createType('GenericAlias<u32>', null);
    expect(encodedNull.toJSON()).toBe(null);
  });
});

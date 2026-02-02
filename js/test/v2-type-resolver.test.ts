import type { Type, TypeDecl } from 'sails-js-types-v2';
import { TypeRegistry } from '@polkadot/types/create';
import { TypeResolver } from '../src/type-resolver-v2.js';

const named = (name: string, generics?: TypeDecl[]): TypeDecl => ({
  kind: 'named',
  name,
  generics,
});

describe('type-resolver-v2 generics', () => {
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
          { name: 'value', type: named('T') },
          { name: 'items', type: { kind: 'slice', item: named('T') } },
        ],
      },
      {
        kind: 'struct',
        name: 'Pair',
        type_params: [{ name: 'T' }, { name: 'U' }],
        fields: [
          { name: 'left', type: named('T') },
          { name: 'right', type: named('Option', [named('U')]) },
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
          { name: 'Some', fields: [{ type: named('T') }] },
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
      a: 'Result<String, u32>',
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
      a: '(String, u32)',
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
      a: '[u32; 3]',
      b: 'u32',
    });

    const encoded = resolver.registry.createType('StructWithArray', { a: [1, 2, 3], b: 123 });
    expect(encoded.toJSON()).toEqual({
      a: [1, 2, 3],
      b: 123,
    });
  });

  test('struct with map', () => {
    const userType: Type = {
      kind: 'struct',
      name: 'StructWithMap',
      fields: [
        { name: 'a', type: named('BTreeMap', ['String', 'u32']) },
        { name: 'b', type: 'u32' },
      ],
    };

    const resolver = new TypeResolver([userType]);

    expect(resolver.getTypeDef(userType)).toEqual({
      a: 'BTreeMap<String, u32>',
      b: 'u32',
    });

    const encoded = resolver.registry.createType('StructWithMap', { a: { foo: 123, bar: 456 }, b: 123 });
    expect(encoded.toJSON()).toEqual({
      a: { foo: 123, bar: 456 },
      b: 123,
    });
  });
});

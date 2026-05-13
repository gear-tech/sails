import { GearApi, HexString, UserMessageSent } from '@gear-js/api';
import { u8aToHex, u8aToU8a } from '@polkadot/util';

import type {
  Type,
  TypeDecl,
  IIdlDoc,
  IServiceExpo,
  IServiceIdent,
  IServiceUnit,
  IFuncParam,
  IServiceEvent,
  DecodedCall,
  DecodedCtor,
  DecodedError,
  DecodedEvent,
  DecodedReply,
  DecodedUnknown,
  DecodeReason,
  ResolvedEntry,
} from './types.js';
import { TransactionBuilderWithHeader } from './transaction-builder-with-header.js';
import { QueryBuilderWithHeader } from './query-builder-with-header.js';
import { SailsMessageHeader, InterfaceId } from './parser.js';
import { TypeResolver } from './type-resolver-idl-v2.js';
import { ZERO_ADDRESS } from './consts.js';

interface ISailsService {
  readonly functions: Record<string, SailsServiceFunc>;
  readonly queries: Record<string, SailsServiceQuery>;
  readonly events: Record<string, ISailsServiceEvent>;
  readonly extends: Record<string, SailsService>;
  readonly routeIdx: number;
  readonly types: ReadonlyMap<string, Type>;
}

interface ISailsFuncArg {
  /** ### Argument name */
  name: string;
  /** ### Argument type */
  type: any;
  /** ### Argument type definition */
  typeDef: TypeDecl;
}

interface ISailsServiceFuncParams {
  /** ### List of argument names and types  */
  readonly args: ISailsFuncArg[];
  /** ### Function return type */
  readonly returnType: any;
  /** ### Function return type definition */
  readonly returnTypeDef: TypeDecl;
  /** ### Encode payload to hex string */
  readonly encodePayload: (...args: any[]) => HexString;
  /** ### Decode payload from hex string */
  readonly decodePayload: <T = any>(bytes: HexString) => T;
  /** ### Decode function result */
  readonly decodeResult: <T = any>(result: HexString) => T;
  /** ### Docs from the IDL file */
  readonly docs?: string;
}

type SailsServiceQuery = ISailsServiceFuncParams & (<T = any>(...args: unknown[]) => QueryBuilderWithHeader<T>);

type SailsServiceFunc = ISailsServiceFuncParams & (<T = any>(...args: unknown[]) => TransactionBuilderWithHeader<T>);

interface ISailsServiceEvent {
  /** ### Event type */
  readonly type: any;
  /** ###  */
  readonly typeDef: IServiceEvent;
  /** ### Check if event is of this type */
  readonly is: (event: UserMessageSent) => boolean;
  /** ### Decode event payload */
  readonly decode: (payload: HexString) => any;
  /** ### Subscribe to event
   * @returns Promise with unsubscribe function
   */
  readonly subscribe: <T = any>(cb: (event: T) => void | Promise<void>) => Promise<() => void>;
  /** ### Docs from the IDL file */
  readonly docs?: string;
}

interface ISailsCtorFuncParams {
  /** ### List of argument names and types  */
  readonly args: ISailsFuncArg[];
  /** ### Encode payload to hex string */
  readonly encodePayload: (...args: any[]) => HexString;
  /** ### Decode payload from hex string */
  readonly decodePayload: <T = any>(bytes: HexString) => T;
  /** ### Create transaction builder from code */
  readonly fromCode: (code: Uint8Array | Buffer, ...args: unknown[]) => TransactionBuilderWithHeader<any>;
  /** ### Create transaction builder from code id */
  readonly fromCodeId: (codeId: HexString, ...args: unknown[]) => TransactionBuilderWithHeader<any>;
  /** ### Docs from the IDL file */
  readonly docs?: string;
}

type RouteEntry = {
  interfaceId: InterfaceId;
  routeIdx: number;
  routeName: string;
  serviceUnit: IServiceUnit;
  resolver: TypeResolver;
};

type InternalFunctionEntry = {
  entry: ResolvedEntry & { kind: 'command' | 'query' };
  resolver: TypeResolver;
  params: ISailsFuncArg[];
  returnType: string;
  throwsType?: string;
};

type InternalEventEntry = {
  entry: ResolvedEntry & { kind: 'event' };
  resolver: TypeResolver;
  type: string;
};

type InternalCtorEntry = {
  entry: ResolvedEntry & { kind: 'ctor' };
  params: ISailsFuncArg[];
};

type InternalEntry = InternalFunctionEntry | InternalEventEntry | InternalCtorEntry;
type ByteInput = Uint8Array<ArrayBufferLike> | HexString;

const _getParamsForTxBuilder = (params: ISailsFuncArg[]) => {
  if (params.length === 0) return null;
  if (params.length === 1) return params[0].type;
  return `(${params.map((p) => p.type).join(', ')})`;
};

const _mapHeaderError = (message: string): DecodeReason | null => {
  if (message.includes('magic')) return 'no-magic';
  if (message.includes('Unsupported Sails version')) return 'bad-version';
  if (message.includes('v1 header must be exactly 16 bytes')) return 'bad-hlen';
  if (message.includes('Header length is less than minimal')) return 'bad-hlen';
  if (message.includes('Reserved byte must be zero in version 1')) return 'bad-reserved';
  if (message.includes('Insufficient bytes for')) return 'too-short';
  return null;
};

const _isDecodedUnknown = (value: unknown): value is DecodedUnknown =>
  typeof value === 'object' && value !== null && (value as DecodedUnknown).kind === 'unknown';

const _isFunctionEntry = (value: InternalEntry): value is InternalFunctionEntry =>
  value.entry.kind === 'command' || value.entry.kind === 'query';

const _getArgsForTxBuilder = (args: any[], params: ISailsFuncArg[]) => {
  if (params.length === 0) return null;
  if (params.length === 1) return args[0];
  return args.slice(0, params.length);
};

/**
 * Collect every `Type` in scope for a service per the self-sufficient service IDL
 * contract: depth-first across the `extends` chain (with cycle detection), then the
 * service's own `types` last so locals shadow base definitions on name collision.
 *
 * Throws on a cyclic `extends` graph (`A → B → A`), reporting the chain.
 * Used by both `SailsService` (to seed its `TypeResolver`) and `SailsProgram`
 * (to build `resolveInService`'s lazy index).
 */
const _collectServiceScopeTypes = (
  service: IServiceUnit,
  resolveServiceUnit?: (ident: IServiceIdent) => IServiceUnit | undefined,
): Type[] => {
  const out: Type[] = [];
  const walk = (unit: IServiceUnit, visited: Set<string>) => {
    if (visited.has(unit.name)) {
      throw new Error(
        `Cyclic service-extends chain detected at "${unit.name}" — chain: ` +
          `${[...visited, unit.name].join(' → ')}`,
      );
    }
    const nextVisited = new Set(visited);
    nextVisited.add(unit.name);

    if (resolveServiceUnit && unit.extends?.length) {
      for (const ident of unit.extends as IServiceIdent[]) {
        const baseUnit = resolveServiceUnit(ident);
        if (!baseUnit) {
          throw new Error(`Service definition for "${ident.name}" not found in IDL`);
        }
        walk(baseUnit, nextVisited);
      }
    }

    for (const t of unit.types ?? []) out.push(t);
  };
  walk(service, new Set());
  return out;
};

const _assertMatchingHeader = (
  payload: Uint8Array | HexString,
  expected: SailsMessageHeader,
  target: string,
) => {
  const { ok, header } = SailsMessageHeader.tryFromBytes(u8aToU8a(payload));
  if (!ok || !header) {
    throw new Error(`Invalid Sails header for ${target}`);
  }

  if (
    header.interfaceId.asU64() !== expected.interfaceId.asU64() ||
    header.entryId !== expected.entryId ||
    // route_idx 0 is the inference sentinel on either side — see docs/sails-header-v1-spec.md §13.6.
    (expected.routeIdx !== 0 && header.routeIdx !== 0 && header.routeIdx !== expected.routeIdx)
  ) {
    throw new Error(
      `Header mismatch for ${target}: expected interface_id=${expected.interfaceId.toString()} ` +
      `entry_id=${expected.entryId} route_idx=${expected.routeIdx}, ` +
      `got interface_id=${header.interfaceId.toString()} ` +
      `entry_id=${header.entryId} route_idx=${header.routeIdx}`,
    );
  }
};

export class SailsProgram {
  private _doc: IIdlDoc;
  private _typeResolver: TypeResolver;
  private _api?: GearApi;
  private _programId?: HexString;
  private _services: Map<bigint, IServiceUnit>;
  private _routes: Map<string, RouteEntry>;
  private _entryCache: Map<string, InternalFunctionEntry | InternalEventEntry>;
  private _ctorsByEntryId: Map<number, InternalCtorEntry>;
  private readonly _programTypes: ReadonlyMap<string, Type>;
  // Lazy index for resolveInService: per-service `Map<typeName, Type>`, pre-merged
  // with the service's transitive extends chain. Populated on first call; not
  // invalidated because `_doc` is immutable after parse.
  private _serviceTypeIndex?: Map<string, Map<string, Type>>;
  private _resolveServiceUnit = (ident: IServiceIdent): IServiceUnit | undefined => {
    if (!ident.interface_id) {
      throw new Error(`Service "${ident.name}" is missing interface_id in IDL`);
    }
    const idu64 = InterfaceId.from(ident.interface_id).asU64();
    return this._services.get(idu64);
  };

  /**
   * ### Crate program from parser IDL document
   * @param doc parser and normalized IDL document
   */
  constructor(doc: IIdlDoc) {
    this._doc = doc;
    if (this._doc.program) {
      this._typeResolver = new TypeResolver(this._doc.program.types ?? []);
    }
    this._programTypes = new Map((doc.program?.types ?? []).map((t) => [t.name, t]));
    this._services = this._initServices();
    this._routes = this._initRoutes();
    this._entryCache = new Map();
    this._ctorsByEntryId = this._initCtors();
  }

  /** ### Set api to use for transactions */
  setApi(api: GearApi) {
    this._api = api;
    return this;
  }

  /** ### Set program id to interact with */
  setProgramId(programId: HexString) {
    this._programId = programId;
    return this;
  }

  /** ### Get program id */
  get programId() {
    return this._programId;
  }

  private _initServices(): Map<bigint, IServiceUnit> {
    const services = new Map<bigint, IServiceUnit>();
    for (const service of this._doc.services ?? []) {
      if (!service.interface_id) {
        throw new Error(`Service "${service.name}" is missing interface_id in IDL`);
      }

      const interfaceId = InterfaceId.from(service.interface_id).asU64();
      services.set(interfaceId, service);
    }

    return services;
  }

  private _initRoutes(): Map<string, RouteEntry> {
    const routes = new Map<string, RouteEntry>();
    for (const expo of this._doc.program?.services ?? []) {
      const serviceUnit = this._resolveServiceUnit(expo);
      if (!serviceUnit) {
        throw new Error(`Service definition for "${expo.name}" not found in IDL`);
      }

      const interfaceId = InterfaceId.from(serviceUnit.interface_id);
      routes.set(this._routeKey(interfaceId, expo.route_idx), {
        interfaceId,
        routeIdx: expo.route_idx,
        routeName: expo.route ?? expo.name,
        serviceUnit,
        resolver: this._resolverForService(serviceUnit),
      });
    }
    return routes;
  }

  private _initCtors(): Map<number, InternalCtorEntry> {
    const ctors = new Map<number, InternalCtorEntry>();
    for (const ctor of this._doc.program?.ctors ?? []) {
      const entryId = ctor.entry_id ?? 0;
      ctors.set(entryId, {
        entry: {
          kind: 'ctor',
          ctor: ctor.name,
          interfaceId: InterfaceId.zero(),
          entryId,
          route_idx: 0,
        },
        params: this._paramsForProgram(ctor.params ?? []),
      });
    }
    return ctors;
  }

  private _routeKey(interfaceId: InterfaceId, routeIdx: number): string {
    return `${interfaceId.asU64()}:${routeIdx}`;
  }

  private _paramsForProgram(params: IFuncParam[]): ISailsFuncArg[] {
    return params.map((p) => ({
      name: p.name,
      type: this._typeResolver.getTypeDeclString(p.type),
      typeDef: p.type,
    }));
  }

  private _resolverForService(service: IServiceUnit): TypeResolver {
    return new TypeResolver(_collectServiceScopeTypes(service, this._resolveServiceUnit));
  }

  /** #### Registry with registered types from the parsed IDL */
  get registry() {
    if (!this._doc.program) {
      throw new Error('Program not exists');
    }

    return this._typeResolver.registry;
  }

  /** #### TypeResolver with registered types from the parsed IDL */
  get typeResolver() {
    if (!this._doc.program) {
      throw new Error('Program not exists');
    }

    return this._typeResolver;
  }

  /**
   * Program-level user types declared in the IDL's `program {…}` block,
   * keyed by type name.
   *
   * These are ambient: visible to every service and to constructors. Returns
   * an empty `Map` when the IDL has no `program {…}` block (services-only
   * IDL) or when the program block has no `types`.
   *
   * Treat as immutable. The `ReadonlyMap` type blocks `.set()` at the type
   * level; runtime mutation is not enforced.
   */
  get programTypes(): ReadonlyMap<string, Type> {
    return this._programTypes;
  }

  /** #### Services with functions and events from the parsed IDL */
  get services(): Record<string, SailsService> {
    const services: Record<string, SailsService> = {};
    const program = this._doc.program;

    if (!program?.services?.length) {
      return services;
    }

    for (const expo of program.services as IServiceExpo[]) {
      const serviceUnit = this._resolveServiceUnit(expo);
      if (!serviceUnit) {
        throw new Error(`Service definition for "${expo.name}" not found in IDL`);
      }
      services[expo.name] = new SailsService(
        serviceUnit,
        this._api,
        this._programId,
        expo.route_idx,
        this._resolveServiceUnit,
      );
    }
    return services;
  }

  /**
   * Resolve a `TypeDecl` to its user-type definition in the scope of a named service.
   *
   * Per the self-sufficient service IDL contract (see `docs/idl-v2-spec.md`), scope is
   * the service's own `types` plus the types of every service it extends, transitively.
   * Service-local definitions shadow base-service definitions on name collision.
   * Program-level `types` are NOT in scope — they belong to program/ctor declarations.
   *
   * Returns `undefined` when the service doesn't exist, the `TypeDecl` isn't a named
   * user type, or the name is not declared in the service's transitive scope.
   *
   * Backed by a lazy `Map`-based index — O(1) per call after the first invocation.
   * Safe to call in tight loops while walking large IDL trees.
   */
  resolveInService(serviceName: string, typeDecl: TypeDecl): Type | undefined {
    if (typeof typeDecl === 'string' || typeDecl.kind !== 'named') return undefined;
    if (!this._serviceTypeIndex) this._buildTypeIndex();
    return this._serviceTypeIndex!.get(serviceName)?.get(typeDecl.name);
  }

  private _buildTypeIndex(): void {
    const serviceIndex = new Map<string, Map<string, Type>>();
    const unitByName = new Map<string, IServiceUnit>();
    for (const unit of this._doc.services ?? []) unitByName.set(unit.name, unit);

    const lookupBase = (ident: IServiceIdent): IServiceUnit | undefined => unitByName.get(ident.name);

    for (const unit of this._doc.services ?? []) {
      const merged = new Map<string, Type>();
      // Locals come last so they win on Map.set last-write-wins.
      for (const t of _collectServiceScopeTypes(unit, lookupBase)) {
        merged.set(t.name, t);
      }
      serviceIndex.set(unit.name, merged);
    }
    this._serviceTypeIndex = serviceIndex;
  }

  private _parseHeader(bytes: ByteInput): { header: SailsMessageHeader; payload: Uint8Array } | DecodedUnknown {
    const payload = u8aToU8a(bytes);
    try {
      const { header } = SailsMessageHeader.tryReadBytes(payload);
      return { header, payload };
    } catch (e) {
      if (e instanceof RangeError) {
        const reason = _mapHeaderError(e.message);
        if (reason) return { kind: 'unknown', reason, detail: e.message };
      }
      throw e;
    }
  }

  private _lookupEntryInRoute(route: RouteEntry, entryId: number): InternalFunctionEntry | InternalEventEntry | undefined {
    const cacheKey = `${this._routeKey(route.interfaceId, route.routeIdx)}:${entryId}`;
    const cached = this._entryCache.get(cacheKey);
    if (cached) return cached;

    const resolver = route.resolver;
    const serviceName = route.serviceUnit.name;
    const routeName = route.routeName;
    const interfaceId = route.interfaceId;

    for (const func of route.serviceUnit.funcs ?? []) {
      const fnEntryId = func.entry_id ?? 0;
      if (fnEntryId !== entryId) continue;

      const entry: InternalFunctionEntry = {
        entry: {
          kind: func.kind,
          service: serviceName,
          fn: func.name,
          route: routeName,
          interfaceId,
          entryId,
          route_idx: route.routeIdx,
        },
        resolver,
        params: this._paramsForService(resolver, func.params ?? []),
        returnType: resolver.getTypeDeclString(func.output),
        throwsType: func.throws ? resolver.getTypeDeclString(func.throws) : undefined,
      };
      this._entryCache.set(cacheKey, entry);
      return entry;
    }

    for (const event of route.serviceUnit.events ?? []) {
      const eventEntryId = event.entry_id ?? 0;
      if (eventEntryId !== entryId) continue;
      const entry: InternalEventEntry = {
        entry: {
          kind: 'event',
          service: serviceName,
          event: event.name,
          route: routeName,
          interfaceId,
          entryId,
          route_idx: route.routeIdx,
        },
        resolver,
        type: event.fields?.length ? (resolver.getStructDef(event.fields, {}, true) as string) : 'Null',
      };
      this._entryCache.set(cacheKey, entry);
      return entry;
    }

    return undefined;
  }

  private _paramsForService(resolver: TypeResolver, params: IFuncParam[]): ISailsFuncArg[] {
    return params.map((p) => ({
      name: p.name,
      type: resolver.getTypeDeclString(p.type),
      typeDef: p.type,
    }));
  }

  private _routeToEntry(header: SailsMessageHeader): InternalFunctionEntry | InternalEventEntry | DecodedUnknown {
    let route: RouteEntry | undefined;
    if (header.routeIdx === 0) {
      const interfaceId = header.interfaceId.asU64();
      const matches = [...this._routes.values()].filter((r) => r.interfaceId.asU64() === interfaceId);
      if (matches.length === 0) return { kind: 'unknown', reason: 'no-service' };
      if (matches.length > 1) return { kind: 'unknown', reason: 'ambiguous-route' };
      route = matches[0];
    } else {
      route = this._routes.get(this._routeKey(header.interfaceId, header.routeIdx));
      if (!route) return { kind: 'unknown', reason: 'no-service' };
    }

    return this._lookupEntryInRoute(route, header.entryId) ?? { kind: 'unknown', reason: 'no-entry' };
  }

  private _ctorToEntry(header: SailsMessageHeader): InternalCtorEntry | DecodedUnknown {
    if (header.interfaceId.asU64() !== 0n || header.routeIdx !== 0) {
      return { kind: 'unknown', reason: 'entry-mismatch' };
    }
    return this._ctorsByEntryId.get(header.entryId) ?? { kind: 'unknown', reason: 'no-entry' };
  }

  private _verifyExpected(header: SailsMessageHeader, expected: ResolvedEntry): boolean {
    return (
      header.interfaceId.asU64() === InterfaceId.from(expected.interfaceId).asU64() &&
      header.entryId === expected.entryId &&
      (header.routeIdx === 0 || expected.route_idx === 0 || header.routeIdx === expected.route_idx)
    );
  }

  private _resolveForDecode(
    header: SailsMessageHeader,
    expected?: ResolvedEntry,
  ): InternalFunctionEntry | InternalEventEntry | DecodedUnknown {
    if (expected && !this._verifyExpected(header, expected)) {
      return { kind: 'unknown', reason: 'entry-mismatch' };
    }
    return this._routeToEntry(header);
  }

  private _decodeWithConsumed<T>(
    resolver: TypeResolver,
    typeName: string,
    bytes: Uint8Array,
    offset: number,
  ): { value: T; consumed: number } | DecodedUnknown {
    try {
      const value = resolver.registry.createType<any>(typeName, bytes.subarray(offset));
      return { value: value.toJSON() as T, consumed: value.encodedLength };
    } catch (e) {
      return { kind: 'unknown', reason: 'decode-failed', detail: String(e) };
    }
  }

  private _checkTrailing(bytes: Uint8Array, offset: number, consumed: number): DecodedUnknown | undefined {
    if (offset + consumed !== bytes.length) {
      return { kind: 'unknown', reason: 'trailing-bytes', consumedLen: consumed };
    }
    return undefined;
  }

  private _decodeArgs(
    resolver: TypeResolver,
    params: ISailsFuncArg[],
    payload: Uint8Array,
    offset: number,
  ): { args: Record<string, unknown>; consumed: number } | DecodedUnknown {
    if (params.length === 0) return { args: {}, consumed: 0 };

    const typeName = params.length === 1 ? params[0].type : `(${params.map((p) => p.type).join(', ')})`;
    const decoded = this._decodeWithConsumed<unknown>(resolver, typeName, payload, offset);
    if (_isDecodedUnknown(decoded)) return decoded;

    const args: Record<string, unknown> = {};
    if (params.length === 1) {
      args[params[0].name] = decoded.value;
    } else {
      const values = decoded.value as unknown[];
      for (const [i, param] of params.entries()) args[param.name] = values[i];
    }
    return { args, consumed: decoded.consumed };
  }

  /** Resolve a Sails header to an IDL entry without decoding the body. */
  resolveEntry(header: SailsMessageHeader): ResolvedEntry | DecodedUnknown {
    if (header.interfaceId.asU64() === 0n) {
      const ctor = this._ctorToEntry(header);
      return _isDecodedUnknown(ctor) ? ctor : ctor.entry;
    }

    const entry = this._routeToEntry(header);
    return _isDecodedUnknown(entry) ? entry : entry.entry;
  }

  /** Return all entries mounted under every route matching an interface id. */
  resolveEntryCandidates(interfaceId: InterfaceId): ResolvedEntry[] {
    const iid = interfaceId.asU64();
    const entries: ResolvedEntry[] = [];
    for (const route of this._routes.values()) {
      if (route.interfaceId.asU64() !== iid) continue;
      for (const func of route.serviceUnit.funcs ?? []) {
        const resolved = this._lookupEntryInRoute(route, func.entry_id ?? 0);
        if (resolved) entries.push(resolved.entry);
      }
      for (const event of route.serviceUnit.events ?? []) {
        const resolved = this._lookupEntryInRoute(route, event.entry_id ?? 0);
        if (resolved) entries.push(resolved.entry);
      }
    }
    return entries;
  }

  /**
   * Decode an untrusted Sails call payload. Construct `SailsProgram` once per CodeId and reuse it.
   * Decoded data is untrusted; do not pass it directly into security-sensitive operations.
   */
  decodeCall(bytes: ByteInput, expectedEntry?: ResolvedEntry): DecodedCall | DecodedUnknown {
    const parsed = this._parseHeader(bytes);
    if (_isDecodedUnknown(parsed)) return parsed;

    const resolved = this._resolveForDecode(parsed.header, expectedEntry);
    if (_isDecodedUnknown(resolved)) return resolved;
    if (!_isFunctionEntry(resolved)) {
      return { kind: 'unknown', reason: 'entry-mismatch' };
    }

    const decoded = this._decodeArgs(resolved.resolver, resolved.params, parsed.payload, parsed.header.hlen);
    if (_isDecodedUnknown(decoded)) return decoded;
    const trailing = this._checkTrailing(parsed.payload, parsed.header.hlen, decoded.consumed);
    if (trailing) return trailing;
    return { kind: 'call', entry: resolved.entry, args: decoded.args };
  }

  /**
   * Decode success reply bytes. The caller must route based on Gear ReplyCode; this method does not
   * infer success or error from SCALE bytes.
   */
  decodeReply(bytes: ByteInput, expectedEntry?: ResolvedEntry): DecodedReply | DecodedUnknown {
    const parsed = this._parseHeader(bytes);
    if (_isDecodedUnknown(parsed)) return parsed;

    const resolved = this._resolveForDecode(parsed.header, expectedEntry);
    if (_isDecodedUnknown(resolved)) return resolved;
    if (!_isFunctionEntry(resolved)) {
      return { kind: 'unknown', reason: 'entry-mismatch' };
    }

    const decoded = this._decodeWithConsumed<unknown>(
      resolved.resolver,
      resolved.returnType,
      parsed.payload,
      parsed.header.hlen,
    );
    if (_isDecodedUnknown(decoded)) return decoded;
    const trailing = this._checkTrailing(parsed.payload, parsed.header.hlen, decoded.consumed);
    if (trailing) return trailing;
    return { kind: 'reply', entry: resolved.entry, result: decoded.value };
  }

  /**
   * Decode error reply bytes for functions declaring `throws`. The caller must route based on Gear
   * ReplyCode; calling this for a success reply will decode the wrong type.
   */
  decodeError(bytes: ByteInput, expectedEntry?: ResolvedEntry): DecodedError | DecodedUnknown {
    const parsed = this._parseHeader(bytes);
    if (_isDecodedUnknown(parsed)) return parsed;

    const resolved = this._resolveForDecode(parsed.header, expectedEntry);
    if (_isDecodedUnknown(resolved)) return resolved;
    if (!_isFunctionEntry(resolved)) {
      return { kind: 'unknown', reason: 'entry-mismatch' };
    }
    if (!resolved.throwsType) return { kind: 'unknown', reason: 'no-throws-type' };

    const decoded = this._decodeWithConsumed<unknown>(
      resolved.resolver,
      resolved.throwsType,
      parsed.payload,
      parsed.header.hlen,
    );
    if (_isDecodedUnknown(decoded)) return decoded;
    const trailing = this._checkTrailing(parsed.payload, parsed.header.hlen, decoded.consumed);
    if (trailing) return trailing;
    return { kind: 'error', entry: resolved.entry, error: decoded.value };
  }

  /** Decode an untrusted Sails event payload. */
  decodeEvent(bytes: ByteInput): DecodedEvent | DecodedUnknown {
    const parsed = this._parseHeader(bytes);
    if (_isDecodedUnknown(parsed)) return parsed;

    const resolved = this._routeToEntry(parsed.header);
    if (_isDecodedUnknown(resolved)) return resolved;
    if (resolved.entry.kind !== 'event') return { kind: 'unknown', reason: 'entry-mismatch' };

    const eventEntry = resolved as InternalEventEntry;
    const decoded = this._decodeWithConsumed<unknown>(
      eventEntry.resolver,
      eventEntry.type,
      parsed.payload,
      parsed.header.hlen,
    );
    if (_isDecodedUnknown(decoded)) return decoded;
    const trailing = this._checkTrailing(parsed.payload, parsed.header.hlen, decoded.consumed);
    if (trailing) return trailing;
    return { kind: 'event', entry: eventEntry.entry, data: decoded.value };
  }

  /** Decode an untrusted constructor payload. */
  decodeCtor(bytes: ByteInput, expectedEntry?: ResolvedEntry): DecodedCtor | DecodedUnknown {
    const parsed = this._parseHeader(bytes);
    if (_isDecodedUnknown(parsed)) return parsed;
    if (expectedEntry && !this._verifyExpected(parsed.header, expectedEntry)) {
      return { kind: 'unknown', reason: 'entry-mismatch' };
    }

    const resolved = this._ctorToEntry(parsed.header);
    if (_isDecodedUnknown(resolved)) return resolved;
    const decoded = this._decodeArgs(this._typeResolver, resolved.params, parsed.payload, parsed.header.hlen);
    if (_isDecodedUnknown(decoded)) return decoded;
    const trailing = this._checkTrailing(parsed.payload, parsed.header.hlen, decoded.consumed);
    if (trailing) return trailing;
    return { kind: 'ctor-call', entry: resolved.entry, args: decoded.args };
  }

  /** #### Constructor functions with arguments from the parsed IDL */
  get ctors(): Record<string, ISailsCtorFuncParams> | null {
    if (!this._doc.program) {
      return null;
    }

    const program = this._doc.program;
    const funcs: Record<string, ISailsCtorFuncParams> = {};

    for (const func of program.ctors) {
      const header = SailsMessageHeader.v1(InterfaceId.zero(), func.entry_id ?? 0, 0);
      const params = func.params.map((p: IFuncParam) => ({
        name: p.name,
        type: this._typeResolver.getTypeDeclString(p.type),
        typeDef: p.type,
      }));
      funcs[func.name] = {
        args: params,
        encodePayload: (...args): HexString => {
          if (args.length !== params.length) {
            throw new Error(`Expected ${params.length} arguments, but got ${args.length}`);
          }

          if (params.length === 0) {
            return u8aToHex(header.toBytes());
          }

          const payload = this.registry.createType(`([u8; 16], ${params.map((p) => p.type).join(', ')})`, [
            header.toBytes(),
            ...args,
          ]);

          return payload.toHex();
        },
        decodePayload: <T = any>(bytes: Uint8Array | HexString) => {
          _assertMatchingHeader(bytes, header, `constructor "${func.name}"`);
          if (params.length === 0) {
            return {} as T;
          }

          const payload = this.registry.createType(`([u8; 16], ${params.map((p) => p.type).join(', ')})`, bytes);
          const result = {} as Record<string, any>;
          for (const [i, param] of params.entries()) {
            result[param.name] = payload[i + 1].toJSON();
          }
          return result as T;
        },
        fromCode: (code: Uint8Array | Buffer, ...args: unknown[]) => {
          if (!this._api) {
            throw new Error('API is not set. Use .setApi method to set API instance');
          }

          const builder = new TransactionBuilderWithHeader(
            this._api,
            this.registry,
            'upload_program',
            header,
            _getArgsForTxBuilder(args, params),
            _getParamsForTxBuilder(params),
            'String',
            code,
          );

          this._programId = builder.programId;
          return builder;
        },
        fromCodeId: (codeId: HexString, ...args: unknown[]) => {
          if (!this._api) {
            throw new Error('API is not set. Use .setApi method to set API instance');
          }

          const builder = new TransactionBuilderWithHeader(
            this._api,
            this.registry,
            'create_program',
            header,
            _getArgsForTxBuilder(args, params),
            _getParamsForTxBuilder(params),
            'String',
            codeId,
          );

          this._programId = builder.programId;
          return builder;
        },
        docs: func.docs ? func.docs.join('\n') : undefined,
      };
    }

    return funcs;
  }

  /** #### Parsed IDL */
  get program() {
    if (!this._doc) {
      throw new Error('IDL is not parsed');
    }

    return this._doc.program;
  }
}

export class SailsService implements ISailsService {
  public readonly functions: Record<string, SailsServiceFunc>;
  public readonly queries: Record<string, SailsServiceQuery>;
  public readonly events: Record<string, ISailsServiceEvent>;

  private _programId?: HexString;
  private _service: IServiceUnit;
  private _typeResolver: TypeResolver;
  private _api?: GearApi;
  private _routeIdx: number;
  private readonly _types: ReadonlyMap<string, Type>;

  private _resolveServiceUnit?: (ident: IServiceIdent) => IServiceUnit | undefined;

  constructor(
    service: IServiceUnit,
    api?: GearApi,
    programId?: HexString,
    routeIdx = 0,
    resolveServiceUnit?: (ident: IServiceIdent) => IServiceUnit | undefined,
  ) {
    this._service = service;
    this._api = api;
    this._programId = programId;
    this._resolveServiceUnit = resolveServiceUnit;
    // Self-contained scope: bases first so locals shadow on collision.
    this._typeResolver = new TypeResolver(_collectServiceScopeTypes(service, resolveServiceUnit));
    this._types = new Map((service.types ?? []).map((t) => [t.name, t]));
    this._routeIdx = routeIdx;

    this.events = this._getEvents(service);
    const { funcs, queries } = this._getFunctions(service);
    this.functions = funcs;
    this.queries = queries;
  }

  /** ### Set api to use for transactions */
  setApi(api: GearApi) {
    this._api = api;
    return this;
  }

  /** ### Set program id to interact with */
  setProgramId(programId: HexString) {
    this._programId = programId;
    return this;
  }

  /** ### Set Service route Idx */
  setRouteIdx(routeIdx: number): SailsService {
    this._routeIdx = routeIdx;
    return this;
  }

  /** ### Get program id */
  get programId() {
    return this._programId;
  }

  /** ### Get program id */
  get routeIdx() {
    return this._routeIdx;
  }

  /** #### Registry with registered types from the ServiceUnit */
  get registry() {
    if (!this._service) {
      throw new Error('Service not set');
    }
    return this._typeResolver.registry;
  }

  /** #### TypeResolver with registered types from the ServiceUnit */
  get typeResolver() {
    return this._typeResolver;
  }

  /**
   * User types declared in *this service's own* IDL `types {…}` block,
   * keyed by type name.
   *
   * Declared-only: this map does not include types inherited from base
   * services via `extends`. To enumerate the transitive scope, walk
   * `.extends[base].types`; to resolve a single `TypeDecl` against the full
   * scope, call `program.resolveInService(serviceName, decl)`.
   *
   * Returns an empty `Map` when the service has no `types {…}` block.
   *
   * Treat as immutable. The `ReadonlyMap` type blocks `.set()` at the type
   * level; runtime mutation is not enforced.
   */
  get types(): ReadonlyMap<string, Type> {
    return this._types;
  }

  private _getFunctions(service: IServiceUnit): {
    funcs: Record<string, SailsServiceFunc>;
    queries: Record<string, SailsServiceQuery>;
  } {
    const funcs: Record<string, SailsServiceFunc> = {};
    const queries: Record<string, SailsServiceQuery> = {};

    for (const func of service.funcs) {
      const entry_id = func.entry_id ?? 0;
      const header = SailsMessageHeader.v1(InterfaceId.from(service.interface_id), entry_id, this.routeIdx);
      const params: ISailsFuncArg[] = func.params.map((p: IFuncParam) => ({
        name: p.name,
        type: this._typeResolver.getTypeDeclString(p.type),
        typeDef: p.type,
      }));
      const returnType = this._typeResolver.getTypeDeclString(func.output);
      if (func.kind == 'query') {
        queries[func.name] = (<T = any>(...args: unknown[]): QueryBuilderWithHeader<T> => {
          if (!this._api) {
            throw new Error('API is not set. Use .setApi method to set API instance');
          }
          if (!this._programId) {
            throw new Error('Program ID is not set. Use .setProgramId method to set program ID');
          }

          return new QueryBuilderWithHeader<T>(
            this._api,
            this.registry,
            this._programId,
            header,
            _getArgsForTxBuilder(args, params),
            _getParamsForTxBuilder(params),
            returnType,
          );
        }) as SailsServiceQuery;
      } else {
        funcs[func.name] = (<T = any>(...args: unknown[]): TransactionBuilderWithHeader<T> => {
          if (!this._api) {
            throw new Error('API is not set. Use .setApi method to set API instance');
          }
          if (!this._programId) {
            throw new Error('Program ID is not set. Use .setProgramId method to set program ID');
          }
          return new TransactionBuilderWithHeader(
            this._api,
            this.registry,
            'send_message',
            header,
            _getArgsForTxBuilder(args, params),
            _getParamsForTxBuilder(params),
            returnType,
            this._programId,
          );
        }) as SailsServiceFunc;
      }

      Object.assign(func.kind == 'query' ? queries[func.name] : funcs[func.name], {
        args: params,
        returnType,
        returnTypeDef: func.output,
        docs: func.docs ? func.docs.join('\n') : undefined,
        encodePayload: (...args: unknown[]): HexString => {
          if (args.length !== params.length) {
            throw new Error(`Expected ${params.length} arguments, but got ${args.length}`);
          }

          if (params.length === 0) {
            return u8aToHex(header.toBytes());
          }

          const payload = this.registry.createType(`([u8; 16], ${params.map((p) => p.type).join(', ')})`, [
            header.toBytes(),
            ...args,
          ]);

          return payload.toHex();
        },
        decodePayload: <T = any>(bytes: HexString) => {
          _assertMatchingHeader(bytes, header, `${service.name}.${func.name}`);
          if (params.length === 0) {
            return {} as T;
          }

          const payload = this.registry.createType(`([u8; 16], ${params.map((p) => p.type).join(', ')})`, bytes);
          const result = {} as Record<string, any>;
          for (const [i, param] of params.entries()) {
            result[param.name] = payload[i + 1].toJSON();
          }
          return result as T;
        },
        decodeResult: <T = any>(result: HexString) => {
          _assertMatchingHeader(result, header, `${service.name}.${func.name} result`);
          const payload = this.registry.createType(`([u8; 16], ${returnType})`, result);
          return payload[1].toJSON() as T;
        },
      });
    }

    return { funcs, queries };
  }

  private _getEvents(service: IServiceUnit): Record<string, ISailsServiceEvent> {
    const events: Record<string, ISailsServiceEvent> = {};
    const interfaceIdu64: bigint = InterfaceId.from(service.interface_id).asU64();
    const expectedRouteIdx = this.routeIdx;
    // route_idx 0 is the inference sentinel on either side — see docs/sails-header-v1-spec.md §13.6.
    const matchesRoute = (received: number) =>
      expectedRouteIdx === 0 || received === 0 || received === expectedRouteIdx;

    for (const event of service.events) {
      const entryId = event.entry_id ?? 0;
      const header = SailsMessageHeader.v1(InterfaceId.from(service.interface_id), entryId, this.routeIdx);
      const typeStr = event.fields?.length ? this._typeResolver.getStructDef(event.fields, {}, true) : 'Null';
      events[event.name] = {
        type: typeStr,
        typeDef: event,
        docs: event.docs ? event.docs.join('\n') : undefined,
        is: ({ data: { message } }: UserMessageSent) => {
          if (!message.destination.eq(ZERO_ADDRESS)) {
            return false;
          }

          const { ok, header } = SailsMessageHeader.tryFromBytes(message.payload);
          if (
            ok &&
            header.interfaceId.asU64() === interfaceIdu64 &&
            header.entryId === entryId &&
            matchesRoute(header.routeIdx)
          ) {
            return true;
          }
          return false;
        },
        decode: (payload: HexString) => {
          _assertMatchingHeader(payload, header, `${service.name}.${event.name}`);
          const data = this.registry.createType(`([u8; 16], ${typeStr})`, payload);
          return data[1].toJSON();
        },
        subscribe: <T = any>(cb: (eventData: T) => void | Promise<void>): Promise<() => void> => {
          if (!this._api) {
            throw new Error('API is not set. Use .setApi method to set API instance');
          }

          if (!this._programId) {
            throw new Error('Program ID is not set. Use .setProgramId method to set program ID');
          }

          return this._api.gearEvents.subscribeToGearEvent('UserMessageSent', ({ data: { message } }) => {
            if (!message.source.eq(this._programId)) return;
            if (!message.destination.eq(ZERO_ADDRESS)) return;

            const { ok, header } = SailsMessageHeader.tryFromBytes(message.payload);
            if (
              ok &&
              header.interfaceId.asU64() === interfaceIdu64 &&
              header.entryId === entryId &&
              matchesRoute(header.routeIdx)
            ) {
              cb(this.registry.createType(`([u8; 16], ${typeStr})`, message.payload)[1].toJSON() as T);
            }
          });
        },
      };
    }

    return events;
  }

  get extends(): Record<string, SailsService> {
    const extended: Record<string, SailsService> = {};

    if (!this._service?.extends?.length || !this._resolveServiceUnit) {
      return extended;
    }

    for (const ident of this._service.extends as IServiceIdent[]) {
      const serviceUnit = this._resolveServiceUnit(ident);
      if (!serviceUnit) {
        throw new Error(`Service definition for "${ident.name}" not found in IDL`);
      }

      extended[ident.name] = new SailsService(
        serviceUnit,
        this._api,
        this._programId,
        this.routeIdx,
        this._resolveServiceUnit,
      );
    }

    return extended;
  }
}

import type { RegisteredProgram } from './registry.js';
import type { IServiceUnit, IServiceFunc, IFuncParam, IServiceEvent } from 'sails-js-types';
import { TypeResolver } from 'sails-js';

function interfaceIdToString(id: any): string | null {
  if (!id) return null;
  if (typeof id === 'string') return id;
  if (typeof id.toString === 'function') return id.toString();
  return null;
}

function summarizeParams(params: IFuncParam[], resolver: TypeResolver) {
  return params.map((p) => ({
    name: p.name,
    type: resolver.getTypeDeclString(p.type),
  }));
}

function summarizeFunc(func: IServiceFunc, idx: number, resolver: TypeResolver) {
  const entryIdAnn = func.annotations?.find(([k]) => k === 'entry-id');
  const entryId = entryIdAnn ? Number(entryIdAnn[1]) : idx;
  return {
    name: func.name,
    kind: func.kind ?? 'command',
    entry_id: entryId,
    params: summarizeParams(func.params ?? [], resolver),
    return_type: func.output && func.output !== '()' ? resolver.getTypeDeclString(func.output) : 'void',
    throws: func.throws ? resolver.getTypeDeclString(func.throws) : undefined,
    docs: func.docs?.join('\n') || undefined,
  };
}

function summarizeEvent(event: IServiceEvent, idx: number, resolver: TypeResolver) {
  const entryIdAnn = event.annotations?.find(([k]) => k === 'entry-id');
  const entryId = entryIdAnn ? Number(entryIdAnn[1]) : idx;
  return {
    name: event.name,
    entry_id: entryId,
    fields: event.fields?.map((f) => ({
      name: f.name,
      type: resolver.getTypeDeclString(f.type),
    })) ?? [],
    docs: event.docs?.join('\n') || undefined,
  };
}

function findServiceUnit(entry: RegisteredProgram, serviceName: string): IServiceUnit {
  const unit = entry.doc.services?.find((s) => s.name === serviceName);
  if (!unit) {
    const available = entry.doc.services?.map((s) => s.name) ?? [];
    throw new Error(
      `Service "${serviceName}" not found in program "${entry.name}". Available: [${available.join(', ')}]`,
    );
  }
  return unit;
}

export function summarizeProgram(entry: RegisteredProgram) {
  const doc = entry.doc;
  const resolver = entry.program.typeResolver;
  return {
    program: entry.name,
    constructors: doc.program?.ctors?.map((c) => ({
      name: c.name,
      params: summarizeParams(c.params ?? [], resolver),
      throws: c.throws ? resolver.getTypeDeclString(c.throws) : undefined,
      docs: c.docs?.join('\n') || undefined,
    })) ?? [],
    services: doc.program?.services?.map((expo) => {
      const unit = doc.services?.find((s) => s.name === expo.name);
      return {
        name: expo.name,
        interface_id: interfaceIdToString(unit?.interface_id ?? expo.interface_id),
        route_idx: expo.route_idx ?? 0,
        function_count: unit?.funcs?.length ?? 0,
        event_count: unit?.events?.length ?? 0,
        extends: unit?.extends?.map((e) => e.name) ?? [],
      };
    }) ?? [],
    type_count: doc.services?.reduce((sum, s) => sum + (s.types?.length ?? 0), 0) ?? 0,
  };
}

export function summarizeService(entry: RegisteredProgram, serviceName: string) {
  const unit = findServiceUnit(entry, serviceName);
  const resolver = entry.program.services[serviceName].typeResolver;
  return {
    name: unit.name,
    interface_id: interfaceIdToString(unit.interface_id),
    functions: unit.funcs?.map((f, i) => summarizeFunc(f, i, resolver)) ?? [],
    events: unit.events?.map((e, i) => summarizeEvent(e, i, resolver)) ?? [],
    types: unit.types?.map((t) => {
      if (t.kind === 'struct') {
        return {
          name: t.name,
          kind: 'struct',
          fields: t.fields.map((f) => ({ name: f.name, type: resolver.getTypeDeclString(f.type) })),
        };
      } else if (t.kind === 'enum') {
        return {
          name: t.name,
          kind: 'enum',
          variants: t.variants.map((v) => v.name),
        };
      } else {
        return {
          name: t.name,
          kind: 'alias',
          target: resolver.getTypeDeclString(t.target),
        };
      }
    }) ?? [],
    extends: unit.extends?.map((e) => ({
      name: e.name,
      interface_id: interfaceIdToString(e.interface_id),
    })) ?? [],
    annotations: unit.annotations ?? [],
  };
}

export function summarizeFunction(
  entry: RegisteredProgram,
  serviceName: string,
  funcName: string,
) {
  const unit = findServiceUnit(entry, serviceName);
  const resolver = entry.program.services[serviceName].typeResolver;
  const allFuncs = unit.funcs ?? [];
  const idx = allFuncs.findIndex((f) => f.name === funcName);
  if (idx === -1) {
    const available = allFuncs.map((f) => f.name);
    throw new Error(
      `Function "${funcName}" not found in service "${serviceName}". Available: [${available.join(', ')}]`,
    );
  }
  const func = allFuncs[idx];
  return {
    ...summarizeFunc(func, idx, resolver),
    service: serviceName,
    interface_id: interfaceIdToString(unit.interface_id),
  };
}

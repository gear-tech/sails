import type { RegisteredProgram } from './registry.js';
import type { IServiceUnit, IServiceFunc, IFuncParam, IServiceEvent, TypeDecl } from 'sails-js-types';

function interfaceIdToString(id: any): string | null {
  if (!id) return null;
  if (typeof id === 'string') return id;
  if (typeof id.toString === 'function') return id.toString();
  return null;
}

function typeDeclToString(td: TypeDecl): string {
  if (!td) return 'unknown';
  if (typeof td === 'string') return td;
  if ('primitive' in td) return td.primitive;
  if ('optional' in td) return `Option<${typeDeclToString(td.optional)}>`;
  if ('vec' in td) return `Vec<${typeDeclToString(td.vec)}>`;
  if ('result' in td) return `Result<${typeDeclToString(td.result.ok)}, ${typeDeclToString(td.result.err)}>`;
  if ('map' in td) return `BTreeMap<${typeDeclToString(td.map.key)}, ${typeDeclToString(td.map.value)}>`;
  if ('fixedArray' in td) return `[${typeDeclToString(td.fixedArray.type)}; ${td.fixedArray.len}]`;
  if ('tuple' in td) return `(${td.tuple.map(typeDeclToString).join(', ')})`;
  if ('userDefined' in td) {
    const name = td.userDefined.name;
    if (td.userDefined.params?.length) {
      return `${name}<${td.userDefined.params.map(typeDeclToString).join(', ')}>`;
    }
    return name;
  }
  return JSON.stringify(td);
}

function summarizeParams(params: IFuncParam[]) {
  return params.map((p) => ({
    name: p.name,
    type: typeDeclToString(p.type),
  }));
}

function summarizeFunc(func: IServiceFunc, idx: number) {
  const entryIdAnn = func.annotations?.find(([k]) => k === 'entry-id');
  const entryId = entryIdAnn ? Number(entryIdAnn[1]) : idx;
  return {
    name: func.name,
    kind: func.kind ?? 'command',
    entry_id: entryId,
    params: summarizeParams(func.params),
    return_type: func.output ? typeDeclToString(func.output) : 'void',
    throws: func.throws ? typeDeclToString(func.throws) : undefined,
    docs: func.docs?.join('\n') || undefined,
  };
}

function summarizeEvent(event: IServiceEvent, idx: number) {
  const entryIdAnn = event.annotations?.find(([k]) => k === 'entry-id');
  const entryId = entryIdAnn ? Number(entryIdAnn[1]) : idx;
  return {
    name: event.name,
    entry_id: entryId,
    fields: event.fields?.map((f) => ({
      name: f.name,
      type: typeDeclToString(f.type),
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
  return {
    program: entry.name,
    constructors: doc.program?.ctors?.map((c, idx) => ({
      name: c.name,
      params: summarizeParams(c.params),
      throws: c.throws ? typeDeclToString(c.throws) : undefined,
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
  return {
    name: unit.name,
    interface_id: interfaceIdToString(unit.interface_id),
    functions: unit.funcs?.map(summarizeFunc) ?? [],
    events: unit.events?.map(summarizeEvent) ?? [],
    types: unit.types?.map((t) => ({
      name: t.name,
      def: t.def,
    })) ?? [],
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
  const allFuncs = unit.funcs ?? [];
  const idx = allFuncs.findIndex((f) => f.name === funcName);
  if (idx === -1) {
    const available = allFuncs.map((f) => f.name);
    throw new Error(
      `Function "${funcName}" not found in service "${serviceName}". Available: [${available.join(', ')}]`,
    );
  }
  const func = allFuncs[idx];
  const detail = summarizeFunc(func, idx);

  // Also include the service context
  return {
    ...detail,
    service: serviceName,
    interface_id: interfaceIdToString(unit.interface_id),
  };
}

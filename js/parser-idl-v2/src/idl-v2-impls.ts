import type {
  AnnotationEntry,
  IDocAnnotated,
  IFuncParam,
  FunctionKind,
  IIdlDoc,
  IProgramUnit,
  IServiceEvent,
  IServiceExpo,
  IServiceFunc,
  IServiceIdent,
  IServiceUnit,
  IStructField,
  Type,
  TypeDecl,
  ITypeEnum,
  ITypeParameter,
  ITypeStruct,
  ITypeAlias,
  ICtorFunc,
  IEnumVariant,
} from 'sails-js-types';

import { InterfaceId } from './interface-id';

const mapArray = <T, U>(items: T[] | undefined, map: (item: T) => U): U[] | undefined =>
  items?.map((item: T) => map(item));

class IdlDoc implements IIdlDoc {
  public readonly globals?: AnnotationEntry[];
  public readonly program?: ProgramUnit;
  public readonly services?: ServiceUnit[];

  constructor(data: IIdlDoc) {
    this.globals = data.globals;
    this.program = data.program ? new ProgramUnit(data.program) : undefined;
    this.services = mapArray(data.services, (service) => new ServiceUnit(service));
  }
}

class ProgramUnit implements IProgramUnit {
  public readonly name: string;
  public readonly ctors?: CtorFunc[];
  public readonly services?: ServiceExpo[];
  public readonly types?: Type[];
  public readonly docs?: string[];
  public readonly annotations?: AnnotationEntry[];

  constructor(data: IProgramUnit) {
    this.name = data.name;
    this.ctors = mapArray(data.ctors, (ctor) => new CtorFunc(ctor));
    this.services = mapArray(data.services, (service) => new ServiceExpo(service));
    this.types = mapArray(data.types, (type) => createType(type));
    this.docs = data.docs;
    this.annotations = data.annotations;
  }
}

class ServiceIdent implements IServiceIdent {
  public readonly name: string;
  public readonly interface_id?: InterfaceId;

  constructor(data: IServiceIdent) {
    this.name = data.name;
    this.interface_id = data.interface_id ? InterfaceId.from(data.interface_id) : undefined;
  }
}

class ServiceExpo extends ServiceIdent implements IServiceExpo {
  public readonly route?: string;
  public readonly route_idx: number;
  public readonly docs?: string[];
  public readonly annotations?: AnnotationEntry[];

  constructor(data: IServiceExpo) {
    super(data);
    this.route = data.route;
    this.route_idx = data.route_idx;
    this.docs = data.docs;
    this.annotations = data.annotations;
  }
}

class CtorFunc implements ICtorFunc {
  public readonly name: string;
  public readonly params?: FuncParam[];
  public readonly entry_id: number;
  public readonly docs?: string[];
  public readonly annotations?: AnnotationEntry[];

  constructor(data: ICtorFunc) {
    this.name = data.name;
    this.params = mapArray(data.params, (param) => new FuncParam(param));
    this.entry_id = data.entry_id ?? 0;
    this.docs = data.docs;
    this.annotations = data.annotations;
  }
}

class ServiceUnit extends ServiceIdent implements IServiceUnit {
  public readonly extends?: ServiceIdent[];
  public readonly funcs?: ServiceFunc[];
  public readonly events?: IServiceEvent[];
  public readonly types?: Type[];
  public readonly docs?: string[];
  public readonly annotations?: AnnotationEntry[];

  constructor(data: IServiceUnit) {
    super(data);
    this.extends = mapArray(data.extends, (ident) => new ServiceIdent(ident));
    this.funcs = mapArray(data.funcs, (func) => new ServiceFunc(func));
    this.events = mapArray(data.events, (event) => new EnumVariant(event));
    this.types = mapArray(data.types, (type) => createType(type));
    this.docs = data.docs;
    this.annotations = data.annotations;
  }
}

class ServiceFunc implements IServiceFunc {
  public readonly name: string;
  public readonly params?: FuncParam[];
  public readonly output: TypeDecl;
  public readonly throws?: TypeDecl;
  public readonly kind: FunctionKind;
  public readonly entry_id: number;
  public readonly docs?: string[];
  public readonly annotations?: AnnotationEntry[];

  constructor(data: IServiceFunc) {
    this.name = data.name;
    this.params = mapArray(data.params, (param) => new FuncParam(param));
    this.output = data.output;
    this.throws = data.throws;
    this.kind = data.kind;
    this.entry_id = data.entry_id ?? 0;
    this.docs = data.docs;
    this.annotations = data.annotations;
  }
}

class FuncParam implements IFuncParam {
  public readonly name: string;
  public readonly type: TypeDecl;

  constructor(data: IFuncParam) {
    this.name = data.name;
    this.type = data.type;
  }
}

class TypeStruct implements ITypeStruct {
  public readonly name: string;
  public readonly type_params?: TypeParameter[];
  public readonly kind: 'struct';
  public readonly fields: StructField[];
  public readonly docs?: string[];
  public readonly annotations?: AnnotationEntry[];

  constructor(data: ITypeStruct) {
    this.name = data.name;
    this.type_params = mapArray(data.type_params, (param) => new TypeParameter(param));
    this.kind = 'struct';
    this.fields = data.fields.map((field: IStructField) => new StructField(field));
    this.docs = data.docs;
    this.annotations = data.annotations;
  }
}

class TypeEnum implements ITypeEnum {
  public readonly name: string;
  public readonly type_params?: TypeParameter[];
  public readonly kind: 'enum';
  public readonly variants: EnumVariant[];
  public readonly docs?: string[];
  public readonly annotations?: AnnotationEntry[];

  constructor(data: ITypeEnum) {
    this.name = data.name;
    this.type_params = mapArray(data.type_params, (param) => new TypeParameter(param));
    this.kind = 'enum';
    this.variants = data.variants.map((variant: IEnumVariant) => new EnumVariant(variant));
    this.docs = data.docs;
    this.annotations = data.annotations;
  }
}

class TypeAlias implements ITypeAlias {
  public readonly name: string;
  public readonly type_params?: TypeParameter[];
  public readonly kind: 'alias';
  public readonly target: TypeDecl;
  public readonly docs?: string[];
  public readonly annotations?: AnnotationEntry[];

  constructor(data: ITypeAlias) {
    this.name = data.name;
    this.type_params = mapArray(data.type_params, (param) => new TypeParameter(param));
    this.kind = 'alias';
    this.target = data.target;
    this.docs = data.docs;
    this.annotations = data.annotations;
  }
}

class TypeParameter implements ITypeParameter {
  public readonly name: string;
  public readonly ty?: TypeDecl;

  constructor(data: ITypeParameter) {
    this.name = data.name;
    this.ty = data.ty;
  }
}

class StructField implements IStructField {
  public readonly name?: string;
  public readonly type: TypeDecl;
  public readonly docs?: string[];
  public readonly annotations?: AnnotationEntry[];

  constructor(data: IStructField) {
    this.name = data.name;
    this.type = data.type;
    this.docs = data.docs;
    this.annotations = data.annotations;
  }
}

class EnumVariant implements IEnumVariant {
  public readonly name: string;
  public readonly fields: StructField[];
  public readonly entry_id: number;
  public readonly docs?: string[];
  public readonly annotations?: AnnotationEntry[];

  constructor(data: IEnumVariant) {
    this.name = data.name;
    this.fields = data.fields.map((field: IStructField) => new StructField(field));
    this.entry_id = data.entry_id ?? 0;
    this.docs = data.docs;
    this.annotations = data.annotations;
  }
}

const createType = (type: Type): Type => {
  if (type.kind === 'struct') {
    return new TypeStruct(type);
  }
  if (type.kind === 'alias') {
    return new TypeAlias(type);
  }

  return new TypeEnum(type);
};

const normalizeDocAnnotated = <T extends IDocAnnotated>(data: T): T => ({
  ...data,
  docs: data.docs ?? [],
  annotations: data.annotations ?? [],
});

const normalizeStructField = (data: IStructField): IStructField => normalizeDocAnnotated(data);

const normalizeEnumVariant = (data: IEnumVariant, fallbackEntryId = 0): IEnumVariant => ({
  ...normalizeDocAnnotated(data),
  fields: (data.fields ?? []).map((data: IStructField) => normalizeStructField(data)),
  entry_id: data.entry_id ?? fallbackEntryId,
});

const normalizeType = (data: Type): Type => {
  const base = normalizeDocAnnotated(data);
  const typeParams = data.type_params ?? [];

  if (data.kind === 'struct') {
    return {
      ...base,
      kind: 'struct',
      type_params: typeParams,
      fields: (data.fields ?? []).map((data: IStructField) => normalizeStructField(data)),
    };
  }

  if (data.kind === 'alias') {
    return {
      ...base,
      kind: 'alias',
      type_params: typeParams,
      target: data.target,
    };
  }

  return {
    ...base,
    kind: 'enum',
    type_params: typeParams,
    variants: (data.variants ?? []).map((data: IEnumVariant) => normalizeEnumVariant(data)),
  };
};

/// Do nothig, leave it for the future-proof
const normalizeFuncParam = (data: IFuncParam): IFuncParam => data;

const normalizeCtorFunc = (data: ICtorFunc, fallbackEntryId: number): ICtorFunc => ({
  ...normalizeDocAnnotated(data),
  params: (data.params ?? []).map((data: IFuncParam) => normalizeFuncParam(data)),
  entry_id: data.entry_id ?? fallbackEntryId,
});

const normalizeServiceIdent = (data: IServiceIdent): IServiceIdent => ({
  ...data,
  interface_id: data.interface_id ? InterfaceId.from(data.interface_id) : undefined,
});

const normalizeServiceExpo = (data: IServiceExpo): IServiceExpo => ({
  ...normalizeDocAnnotated(data),
  interface_id: data.interface_id ? InterfaceId.from(data.interface_id) : undefined,
});

const normalizeServiceFunc = (data: IServiceFunc, fallbackEntryId: number): IServiceFunc => ({
  ...normalizeDocAnnotated(data),
  params: (data.params ?? []).map((data: IFuncParam) => normalizeFuncParam(data)),
  entry_id: data.entry_id ?? fallbackEntryId,
});

const normalizeServiceUnit = (data: IServiceUnit): IServiceUnit => ({
  ...normalizeDocAnnotated(data),
  interface_id: data.interface_id ? InterfaceId.from(data.interface_id) : undefined,
  extends: (data.extends ?? []).map((data: IServiceIdent) => normalizeServiceIdent(data)),
  funcs: (data.funcs ?? []).map((data: IServiceFunc, idx: number) => normalizeServiceFunc(data, idx)),
  events: (data.events ?? []).map((data: IEnumVariant, idx: number) => normalizeEnumVariant(data, idx)),
  types: (data.types ?? []).map((data: Type) => normalizeType(data)),
});

const normalizeProgramUnit = (data: IProgramUnit): IProgramUnit => ({
  ...normalizeDocAnnotated(data),
  ctors: (data.ctors ?? []).map((data: ICtorFunc, idx: number) => normalizeCtorFunc(data, idx)),
  services: (data.services ?? []).map((data: IServiceExpo) => normalizeServiceExpo(data)),
  types: (data.types ?? []).map((data: Type) => normalizeType(data)),
});

export const normalizeIdl = (data: IIdlDoc): IIdlDoc =>
  new IdlDoc({
    globals: data.globals ?? [],
    program: data.program ? normalizeProgramUnit(data.program) : undefined,
    services: (data.services ?? []).map((data: IServiceUnit) => normalizeServiceUnit(data)),
  });

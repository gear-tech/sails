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
  ICtorFunc,
  IEnumVariant,
} from "sails-js-types-v2";

import { InterfaceId } from "./interface_id";

const mapArray = <T, U>(items: T[] | undefined, map: (item: T) => U): U[] | undefined =>
  items?.map(map);

export class IdlDoc implements IIdlDoc {
  public readonly globals?: AnnotationEntry[];
  public readonly program?: ProgramUnit;
  public readonly services?: ServiceUnit[];

  constructor(data: IIdlDoc) {
    this.globals = data.globals;
    this.program = data.program ? new ProgramUnit(data.program) : undefined;
    this.services = mapArray(data.services, (service) => new ServiceUnit(service));
  }
}

export class ProgramUnit implements IProgramUnit {
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

export class ServiceIdent implements IServiceIdent {
  public readonly name: string;
  public readonly interface_id?: InterfaceId;

  constructor(data: IServiceIdent) {
    this.name = data.name;
    this.interface_id = data.interface_id ? InterfaceId.from(data.interface_id) : undefined;
  }
}

export class ServiceExpo extends ServiceIdent implements IServiceExpo {
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

export class CtorFunc implements ICtorFunc {
  public readonly name: string;
  public readonly params?: FuncParam[];
  public readonly docs?: string[];
  public readonly annotations?: AnnotationEntry[];

  constructor(data: ICtorFunc) {
    this.name = data.name;
    this.params = mapArray(data.params, (param) => new FuncParam(param));
    this.docs = data.docs;
    this.annotations = data.annotations;
  }
}

export class ServiceUnit extends ServiceIdent implements IServiceUnit {
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

export class ServiceFunc implements IServiceFunc {
  public readonly name: string;
  public readonly params?: FuncParam[];
  public readonly output: TypeDecl;
  public readonly throws?: TypeDecl;
  public readonly kind: FunctionKind;
  public readonly docs?: string[];
  public readonly annotations?: AnnotationEntry[];

  constructor(data: IServiceFunc) {
    this.name = data.name;
    this.params = mapArray(data.params, (param) => new FuncParam(param));
    this.output = data.output;
    this.throws = data.throws;
    this.kind = data.kind;
    this.docs = data.docs;
    this.annotations = data.annotations;
  }
}

export class FuncParam implements IFuncParam {
  public readonly name: string;
  public readonly type: TypeDecl;

  constructor(data: IFuncParam) {
    this.name = data.name;
    this.type = data.type;
  }
}

export class TypeStruct implements ITypeStruct {
  public readonly name: string;
  public readonly type_params?: TypeParameter[];
  public readonly kind: "struct";
  public readonly fields: StructField[];
  public readonly docs?: string[];
  public readonly annotations?: AnnotationEntry[];

  constructor(data: ITypeStruct) {
    this.name = data.name;
    this.type_params = mapArray(data.type_params, (param) => new TypeParameter(param));
    this.kind = "struct";
    this.fields = data.fields.map((field) => new StructField(field));
    this.docs = data.docs;
    this.annotations = data.annotations;
  }
}

export class TypeEnum implements ITypeEnum {
  public readonly name: string;
  public readonly type_params?: TypeParameter[];
  public readonly kind: "enum";
  public readonly variants: EnumVariant[];
  public readonly docs?: string[];
  public readonly annotations?: AnnotationEntry[];

  constructor(data: ITypeEnum) {
    this.name = data.name;
    this.type_params = mapArray(data.type_params, (param) => new TypeParameter(param));
    this.kind = "enum";
    this.variants = data.variants.map((variant) => new EnumVariant(variant));
    this.docs = data.docs;
    this.annotations = data.annotations;
  }
}

export class TypeParameter implements ITypeParameter {
  public readonly name: string;
  public readonly ty?: TypeDecl;

  constructor(data: ITypeParameter) {
    this.name = data.name;
    this.ty = data.ty;
  }
}

export class StructField implements IStructField {
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

export class EnumVariant implements IEnumVariant {
  public readonly name: string;
  public readonly fields: StructField[];
  public readonly docs?: string[];
  public readonly annotations?: AnnotationEntry[];

  constructor(data: IEnumVariant) {
    this.name = data.name;
    this.fields = data.fields.map((field) => new StructField(field));
    this.docs = data.docs;
    this.annotations = data.annotations;
  }
}

export const createType = (type: Type): Type => {
  if (type.kind === "struct") {
    return new TypeStruct(type);
  }

  return new TypeEnum(type);
};

const normalizeDocAnnotated = <T extends IDocAnnotated>(data: T): T => ({
  ...data,
  docs: data.docs ?? [],
  annotations: data.annotations ?? [],
});

const normalizeStructField = (data: IStructField): IStructField => ({
  ...normalizeDocAnnotated(data),
});

const normalizeEnumVariant = (data: IEnumVariant): IEnumVariant => ({
  ...normalizeDocAnnotated(data),
  fields: (data.fields ?? []).map(normalizeStructField),
});

const normalizeType = (data: Type): Type => {
  const base = normalizeDocAnnotated(data);
  const typeParams = (data.type_params ?? []).map((param) => ({
    ...param,
  }));

  if (data.kind === "struct") {
    return {
      ...base,
      kind: "struct",
      type_params: typeParams,
      fields: (data.fields ?? []).map(normalizeStructField),
    };
  }

  return {
    ...base,
    kind: "enum",
    type_params: typeParams,
    variants: (data.variants ?? []).map(normalizeEnumVariant),
  };
};

const normalizeFuncParam = (data: IFuncParam): IFuncParam => ({
  ...data,
});

const normalizeCtorFunc = (data: ICtorFunc): ICtorFunc => ({
  ...normalizeDocAnnotated(data),
  params: (data.params ?? []).map(normalizeFuncParam),
});

const normalizeServiceIdent = (data: IServiceIdent): IServiceIdent => ({
  ...data,
  interface_id: data.interface_id ? InterfaceId.from(data.interface_id) : undefined,
});

const normalizeServiceExpo = (data: IServiceExpo): IServiceExpo => ({
  ...normalizeDocAnnotated(data),
  interface_id: data.interface_id ? InterfaceId.from(data.interface_id) : undefined,
});

const normalizeServiceFunc = (data: IServiceFunc): IServiceFunc => ({
  ...normalizeDocAnnotated(data),
  params: (data.params ?? []).map(normalizeFuncParam),
});

const normalizeServiceUnit = (data: IServiceUnit): IServiceUnit => ({
  ...normalizeDocAnnotated(data),
  interface_id: data.interface_id ? InterfaceId.from(data.interface_id) : undefined,
  extends: (data.extends ?? []).map(normalizeServiceIdent),
  funcs: (data.funcs ?? []).map(normalizeServiceFunc),
  events: (data.events ?? []).map(normalizeEnumVariant),
  types: (data.types ?? []).map(normalizeType),
});

const normalizeProgramUnit = (data: IProgramUnit): IProgramUnit => ({
  ...normalizeDocAnnotated(data),
  ctors: (data.ctors ?? []).map(normalizeCtorFunc),
  services: (data.services ?? []).map(normalizeServiceExpo),
  types: (data.types ?? []).map(normalizeType),
});

export const fromJson = (data: IIdlDoc): IdlDoc =>
  new IdlDoc({
    globals: data.globals ?? [],
    program: data.program ? normalizeProgramUnit(data.program) : undefined,
    services: (data.services ?? []).map(normalizeServiceUnit),
  });

export interface IDocAnnotated {
  docs?: string[];
  annotations?: AnnotationEntry[];
}

export type AnnotationEntry = [string, string | null];
export type InterfaceIdInput = string | Uint8Array | ArrayLike<number> | number | bigint | IInterfaceId;

export interface IInterfaceId {
  bytes: Uint8Array;
}

export interface IIdlDoc {
  globals?: AnnotationEntry[];
  program?: IProgramUnit;
  services?: IServiceUnit[];
}

export interface IProgramUnit extends IDocAnnotated {
  name: string;
  ctors?: ICtorFunc[];
  services?: IServiceExpo[];
  types?: Type[];
}

export interface IServiceIdent {
  name: string;
  interface_id?: InterfaceIdInput;
}

export interface IServiceExpo extends IServiceIdent, IDocAnnotated {
  route?: string;
  route_idx: number;
}

export interface ICtorFunc extends IDocAnnotated {
  name: string;
  params?: IFuncParam[];
}

export interface IServiceUnit extends IServiceIdent, IDocAnnotated {
  extends?: IServiceIdent[];
  funcs?: IServiceFunc[];
  events?: IServiceEvent[];
  types?: Type[];
}

export interface IServiceFunc extends IDocAnnotated {
  name: string;
  params?: IFuncParam[];
  output: TypeDecl;
  throws?: TypeDecl;
  kind: FunctionKind;
}

export type FunctionKind = 'command' | 'query';

export interface IFuncParam {
  name: string;
  type: TypeDecl;
}

export type IServiceEvent = IEnumVariant;

// Type declarations
export type TypeDecl = PrimitiveType | ITypeDeclSlice | ITypeDeclArray | ITypeDeclTuple | ITypeDeclNamed;

export interface ITypeDeclSlice {
  kind: 'slice';
  item: TypeDecl;
}

export interface ITypeDeclArray {
  kind: 'array';
  item: TypeDecl;
  len: number;
}

export interface ITypeDeclTuple {
  kind: 'tuple';
  types: TypeDecl[];
}

export interface ITypeDeclNamed {
  kind: 'named';
  name: string;
  generics?: TypeDecl[];
}

// PrimitiveType is encoded as a string (no `kind` field).
export type PrimitiveType =
  | '()'
  | 'bool'
  | 'char'
  | 'String'
  | 'u8'
  | 'u16'
  | 'u32'
  | 'u64'
  | 'u128'
  | 'i8'
  | 'i16'
  | 'i32'
  | 'i64'
  | 'i128'
  | 'ActorId'
  | 'CodeId'
  | 'MessageId'
  | 'H160'
  | 'H256'
  | 'U256';

export type Type = ITypeStruct | ITypeEnum;

export interface ITypeBase extends IDocAnnotated {
  name: string;
  type_params?: ITypeParameter[];
}

export interface ITypeStruct extends ITypeBase {
  kind: 'struct';
  fields: IStructField[];
}

export interface ITypeEnum extends ITypeBase {
  kind: 'enum';
  variants: IEnumVariant[];
}

export interface ITypeParameter {
  name: string;
  ty?: TypeDecl;
}

export interface IStructField extends IDocAnnotated {
  name?: string;
  type: TypeDecl;
}

export interface IEnumVariant extends IDocAnnotated {
  name: string;
  fields: IStructField[];
}

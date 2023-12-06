export interface IGenericTypeDecl {
  name: string;
  kind: 'generic';
  generic: IInnerType[];
}

export interface ISimpleTypeDecl {
  name: string;
  kind: 'simple';
}

export type ITypeDecl = IGenericTypeDecl | ISimpleTypeDecl;

interface IBaseType {
  type: ITypeDecl;
}

export interface ITypeNameType extends IBaseType {
  def: ITypeDecl;
  kind: 'typeName';
}

export interface IOptionType extends IBaseType {
  def: ITypeNameType | IOptionType;
  kind: 'option';
}

export interface IResultType extends IBaseType {
  def: { ok: ITypeNameType | IOptionType; err: ITypeNameType | IOptionType };
  kind: 'result';
}

export interface IVecType extends IBaseType {
  def: ITypeNameType;
  kind: 'vec';
}

export interface ITupleType extends IBaseType {
  def: {
    fields: IInnerType[];
  };
  kind: 'tuple';
}

export interface IStructFieldDef {
  name: string;
  type: IInnerType;
}

export interface IStructType extends IBaseType {
  def: {
    fields: IStructFieldDef[];
  };
  kind: 'struct';
}

export interface IVariantField {
  name: string;
  type?: IInnerType | Omit<IStructType, 'type'>;
}

export interface IVariantType extends IBaseType {
  def: {
    variants: IVariantField[];
  };
  kind: 'variant';
}

export type IInnerType =
  | Omit<ITypeNameType, 'type'>
  | Omit<IOptionType, 'type'>
  | Omit<IResultType, 'type'>
  | Omit<IVecType, 'type'>
  | Omit<ITupleType, 'type'>
  | Omit<IStructType, 'type'>;

export type IType = ITypeNameType | IOptionType | IResultType | IVecType | ITupleType | IStructType | IVariantType;

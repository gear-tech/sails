import { ISailsDocs } from './idl';
import { ISailsEnumVariant, ISailsType, ISailsTypeDef } from './types';

export interface ISailsProgram {
  readonly services: ISailsService[];
  readonly ctor: ISailsCtor;
  readonly types: ISailsType[];

  getTypeByName(name: string): ISailsType;
  getType(id: number): ISailsType;
}

export interface ISailsService {
  readonly funcs: ISailsServiceFunc[];
  readonly events: ISailsServiceEvent[];
  readonly name: string;
}

export interface ISailsServiceFunc extends ISailsDocs {
  readonly name: string;
  readonly isQuery: boolean;
  readonly params: ISailsFuncParam[];
  readonly def: ISailsTypeDef;
}

export interface ISailsCtor {
  readonly funcs: ISailsCtorFunc[];
}

export interface ISailsCtorFunc extends ISailsDocs {
  readonly name: string;
  readonly params: ISailsFuncParam[];
}

export interface ISailsFuncParam {
  readonly name: string;
  readonly def: ISailsTypeDef;
}

export type ISailsServiceEvent = ISailsEnumVariant;

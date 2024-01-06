import { IInnerType } from './types';

export interface IArgument {
  name: string;
  type: IInnerType;
}

export type MethodType = 'message' | 'query';

export interface IServiceMethodDef {
  name: string;
  args: IArgument[];
  output: IInnerType;
}

export interface IServiceMethod {
  kind: MethodType;
  def: IServiceMethodDef;
}

export interface IService {
  name?: string;
  methods: IServiceMethod[];
}

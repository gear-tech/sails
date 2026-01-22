import type { IIdlDoc } from './idl-v2-types';

export interface IIdlParser {
  init(): Promise<void>;
  parse(idl: string): IIdlDoc;
}

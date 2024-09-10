import { ISailsProgram } from './program';

export interface ISailsIdlParser {
  init(): Promise<void>;
  parse(idl: string): ISailsProgram;
}

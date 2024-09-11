import { ISailsProgram } from 'sails-js-types';

import { Output } from './output.js';
import { BaseGenerator } from './base.js';

export class HooksGenerator extends BaseGenerator {
  constructor(out: Output, private _program: ISailsProgram) {
    super(out);
  }

  private generateUseProgramReturn = () => {
    this._out.line('return useGearProgram({ library: Program, id })');
  };

  private generateUseSendTransaction = (serviceName: string, functionName: string) => {
    const name = `useSend${serviceName}${functionName}Transaction`;

    this._out
      .block(`export function ${name}(programId: HexString | undefined)`, () => {
        this._out
          .line('const { data: program } = useProgram(programId)')
          .line()
          .line(
            `return useSendProgramTransaction({ program, serviceName: '${serviceName}', functionName: '${functionName}' })`,
          );
      })
      .line();
  };

  private generateUseQuery = (serviceName: string, functionName: string) => {
    const name = `use${serviceName}${functionName}Query`;

    this._out
      // TODO: args type
      .block(`export function ${name}(programId: HexString | undefined, args: any)`, () => {
        this._out
          .line('const { data: program } = useProgram(programId)')
          .line()
          .line(
            `return useProgramQuery({ program, serviceName: '${serviceName}', functionName: '${functionName}', args })`,
          );
      })
      .line();
  };

  public generate() {
    const LIB_FILE_NAME = 'lib'; // TODO: pass file name

    this._out
      .import('@gear-js/api', 'HexString')
      .import('@gear-js/react-hooks', 'useProgram as useGearProgram, useSendProgramTransaction, useProgramQuery')
      .import(`./${LIB_FILE_NAME}`, 'Program')
      .block(`export function useProgram(id: HexString | undefined)`, this.generateUseProgramReturn)
      .line();

    for (const service of this._program.services) {
      for (const { isQuery, name } of service.funcs) {
        if (isQuery) {
          this.generateUseQuery(service.name, name);
        } else {
          this.generateUseSendTransaction(service.name, name);
        }
      }
    }
  }
}

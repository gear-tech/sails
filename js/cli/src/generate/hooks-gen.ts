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

  public generate() {
    const LIB_FILE_NAME = 'lib'; // TODO: pass file name

    this._out
      .import('@gear-js/api', 'HexString')
      .import('@gear-js/react-hooks', 'useProgram as useGearProgram, useSendProgramTransaction')
      .import(`./${LIB_FILE_NAME}`, 'Program')
      .block(`export function useProgram(id: HexString | undefined)`, this.generateUseProgramReturn)
      .line();

    Object.values(this._program.services).forEach(({ funcs, ...service }) => {
      funcs.forEach((func) => {
        if (func.isQuery) return;

        this._out
          .block(
            `export function useSend${service.name}${func.name}Transaction(programId: HexString | undefined)`,
            () => {
              this._out
                .line('const { data: program } = useProgram(programId)')
                .line()
                .line(
                  `return useSendProgramTransaction({ program, serviceName: '${service.name}', functionName: '${func.name}' })`,
                );
            },
          )
          .line();
      });
    });
  }
}

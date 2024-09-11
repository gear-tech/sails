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
    const libFileName = 'lib'; // TODO: pass file name

    this._out
      .import('@gear-js/api', 'HexString')
      .import('@gear-js/react-hooks', 'useProgram as useGearProgram')
      .import(`./${libFileName}`, 'Program')
      .block(`export function useProgram(id: HexString | undefined)`, this.generateUseProgramReturn)
      .line();
  }
}

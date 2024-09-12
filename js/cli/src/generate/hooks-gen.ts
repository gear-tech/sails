import { ISailsProgram } from 'sails-js-types';
import { toLowerCaseFirst } from 'sails-js-util';

import { Output } from './output.js';
import { BaseGenerator } from './base.js';

export class HooksGenerator extends BaseGenerator {
  constructor(out: Output, private _program: ISailsProgram) {
    super(out);
  }

  private generateUseSendTransaction = (serviceName: string, functionName: string) => {
    const name = `useSend${serviceName}${functionName}Transaction`;

    this._out
      .block(`export function ${name}(programId: HexString | undefined)`, () => {
        this._out
          .line('const { data: program } = useProgram(programId)')
          .line()
          .line(
            `return useSendProgramTransaction({ program, serviceName: '${toLowerCaseFirst(
              serviceName,
            )}', functionName: '${toLowerCaseFirst(functionName)}' })`,
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
            `return useProgramQuery({ program, serviceName: '${toLowerCaseFirst(
              serviceName,
            )}', functionName: '${toLowerCaseFirst(functionName)}', args })`,
          );
      })
      .line();
  };

  private generateUseEvent = (serviceName: string, eventName: string) => {
    const name = `use${serviceName}${eventName}Event`;

    this._out
      // TODO: onData type
      .block(`export function ${name}(programId: HexString | undefined, onData: any)`, () => {
        this._out
          .line('const { data: program } = useProgram(programId)')
          .line()
          .line(
            `return useProgramEvent({ program, serviceName: '${toLowerCaseFirst(
              serviceName,
            )}', functionName: 'subscribeTo${eventName}Event', onData })`,
          );
      })
      .line();
  };

  public generate() {
    const { services } = this._program;
    const LIB_FILE_NAME = 'lib'; // TODO: pass file name

    this._out
      .import('@gear-js/api', 'HexString')
      .import(
        '@gear-js/react-hooks',
        'useProgram as useSailsProgram, useSendProgramTransaction, useProgramQuery, useProgramEvent',
      )
      .import(`./${LIB_FILE_NAME}`, 'Program')
      .block('export function useProgram(id: HexString | undefined)', () =>
        this._out.line('return useSailsProgram({ library: Program, id })'),
      )
      .line();

    Object.values(services).forEach(({ funcs, events, ...service }) => {
      funcs.forEach(({ isQuery, name }) =>
        (isQuery ? this.generateUseQuery : this.generateUseSendTransaction)(service.name, name),
      );

      events.forEach(({ name }) => this.generateUseEvent(service.name, name));
    });
  }
}

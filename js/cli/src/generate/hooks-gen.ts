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
    const formattedServiceName = toLowerCaseFirst(serviceName);
    const formattedFunctionName = toLowerCaseFirst(functionName);

    this._out
      // TODO: rest parameters
      .block(
        `export function ${name}(programId: HexString | undefined, args: QueryArgs<InstanceType<typeof Program>['${formattedServiceName}']['${formattedFunctionName}']>)`,
        () => {
          this._out
            .line('const { data: program } = useProgram(programId)')
            .line()
            .line(
              `return useProgramQuery({ program, serviceName: '${formattedServiceName}', functionName: '${formattedFunctionName}', args })`,
            );
        },
      )
      .line();
  };

  private generateUseEvent = (serviceName: string, eventName: string) => {
    const name = `use${serviceName}${eventName}Event`;
    const formattedServiceName = toLowerCaseFirst(serviceName);
    const functionName = `subscribeTo${eventName}Event`;

    this._out
      // TODO: rest parameters
      .block(
        `export function ${name}(programId: HexString | undefined, onData: (...args: EventCallbackArgs<Event<InstanceType<typeof Program>['${formattedServiceName}']['${functionName}']>>) => void)`,
        () => {
          this._out
            .line('const { data: program } = useProgram(programId)')
            .line()
            .line(
              `return useProgramEvent({ program, serviceName: '${formattedServiceName}', functionName: '${functionName}', onData })`,
            );
        },
      )
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
      // TODO: combine with above after hooks update
      .import('@gear-js/react-hooks/dist/esm/hooks/sails/types', 'Event, EventCallbackArgs, QueryArgs')
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

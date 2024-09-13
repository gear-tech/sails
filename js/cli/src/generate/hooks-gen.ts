import { ISailsProgram } from 'sails-js-types';
import { toLowerCaseFirst } from 'sails-js-util';

import { Output } from './output.js';
import { BaseGenerator } from './base.js';

// type UseProgramParameters = Omit<useSailsProgramParameters<Program>, 'library'>;

// type ProgramType = InstanceType<typeof Program>;
// type ServiceName = SailsServiceName<ProgramType>;

// type UseQueryParameters<
//   TServiceName extends ServiceName,
//   TFunctionName extends QueryName<ProgramType[TServiceName]>,
// > = Omit<
//   UseProgramQueryParameters<
//     ProgramType,
//     TServiceName,
//     TFunctionName,
//     QueryArgs<ProgramType[TServiceName][TFunctionName]>,
//     QueryReturn<ProgramType[TServiceName][TFunctionName]>
//   >,
//   'program' | 'serviceName' | 'functionName'
// >;

// type UseEventParameters<
//   TServiceName extends ServiceName,
//   TFunctionName extends FunctionName<ProgramType[TServiceName], EventReturn>,
// > = Omit<
//   UseProgramEventParameters<
//     ProgramType,
//     TServiceName,
//     TFunctionName,
//     EventCallbackArgs<Event<ProgramType[TServiceName][TFunctionName]>>
//   >,
//   'program' | 'serviceName' | 'functionName'
// >;

export class HooksGenerator extends BaseGenerator {
  constructor(out: Output, private _program: ISailsProgram) {
    super(out);
  }

  private generateTypes = () => {
    this._out
      .line("type UseProgramParameters = Omit<useSailsProgramParameters<Program>, 'library'>")
      .line('type ProgramType = InstanceType<typeof Program>')
      .line('type ServiceName = SailsServiceName<ProgramType>')
      .line()
      .line('type UseQueryParameters<', false)
      .line('  TServiceName extends ServiceName,', false)
      .line('  TFunctionName extends QueryName<ProgramType[TServiceName]>,', false)
      .line('> = Omit<', false)
      .line('  UseProgramQueryParameters<', false)
      .line('    ProgramType,', false)
      .line('    TServiceName,', false)
      .line('    TFunctionName,', false)
      .line('    QueryArgs<ProgramType[TServiceName][TFunctionName]>,', false)
      .line('    QueryReturn<ProgramType[TServiceName][TFunctionName]>', false)
      .line('  >,', false)
      .line("  'program' | 'serviceName' | 'functionName'", false)
      .line('>')
      .line()
      .line('type UseEventParameters<', false)
      .line('  TServiceName extends ServiceName,', false)
      .line('  TFunctionName extends FunctionName<ProgramType[TServiceName], EventReturn>,', false)
      .line('> = Omit<', false)
      .line('  UseProgramEventParameters<', false)
      .line('    ProgramType,', false)
      .line('    TServiceName,', false)
      .line('    TFunctionName,', false)
      .line('    EventCallbackArgs<Event<ProgramType[TServiceName][TFunctionName]>>', false)
      .line('  >,', false)
      .line("  'program' | 'serviceName' | 'functionName'", false)
      .line('>;');
  };

  private generateUseSendTransaction = (serviceName: string, functionName: string) => {
    const name = `useSend${serviceName}${functionName}Transaction`;

    this._out
      .block(`export function ${name}(programId: HexString | undefined)`, () => {
        this._out
          .line('const { data: program } = useProgram({ id: programId })')
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
      .block(
        `export function ${name}(programId: HexString | undefined, parameters: UseQueryParameters<'${formattedServiceName}', '${formattedFunctionName}'>)`,
        () => {
          this._out
            .line('const { data: program } = useProgram({ id: programId })')
            .line()
            .line(
              `return useProgramQuery({ program, serviceName: '${formattedServiceName}', functionName: '${formattedFunctionName}', ...parameters })`,
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
        `export function ${name}(programId: HexString | undefined, parameters: UseEventParameters<'${formattedServiceName}', '${functionName}'>)`,
        () => {
          this._out
            .line('const { data: program } = useProgram({ id: programId })')
            .line()
            .line(
              `return useProgramEvent({ program, serviceName: '${formattedServiceName}', functionName: '${functionName}', ...parameters })`,
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
        'useProgram as useSailsProgram, useSendProgramTransaction, useProgramQuery, useProgramEvent, UseProgramParameters as useSailsProgramParameters, UseProgramQueryParameters, UseProgramEventParameters',
      )
      // TODO: combine with above after hooks update
      .import(
        '@gear-js/react-hooks/dist/esm/hooks/sails/types',
        'Event, EventCallbackArgs, QueryArgs, ServiceName as SailsServiceName, FunctionName, QueryName, QueryReturn, EventReturn',
      )
      .import(`./${LIB_FILE_NAME}`, 'Program')
      .block('export function useProgram({ id, ...parameters }: UseProgramParameters)', () =>
        this._out.line('return useSailsProgram({ library: Program, id, ...parameters })'),
      )
      .line();

    this.generateTypes();

    Object.values(services).forEach(({ funcs, events, ...service }) => {
      funcs.forEach(({ isQuery, name }) =>
        (isQuery ? this.generateUseQuery : this.generateUseSendTransaction)(service.name, name),
      );

      events.forEach(({ name }) => this.generateUseEvent(service.name, name));
    });
  }
}

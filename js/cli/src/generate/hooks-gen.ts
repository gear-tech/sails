import { ISailsProgram } from 'sails-js-types';
import { toLowerCaseFirst } from 'sails-js-util';

import { Output } from './output.js';
import { BaseGenerator } from './base.js';

export class HooksGenerator extends BaseGenerator {
  constructor(out: Output, private _program: ISailsProgram) {
    super(out);
  }

  private generateImports = () => {
    const LIB_FILE_NAME = 'lib'; // TODO: pass file name

    this._out
      .import(
        '@gear-js/react-hooks',
        `useProgram as useSailsProgram,
        useSendProgramTransaction,
        usePrepareProgramTransaction,
        useProgramQuery,
        useProgramEvent,
        UseProgramParameters as useSailsProgramParameters,
        UseProgramQueryParameters,
        UseProgramEventParameters`,
      )
      // TODO: combine with above after hooks update
      .import(
        '@gear-js/react-hooks/dist/esm/hooks/sails/types',
        `Event,
        EventCallbackArgs,
        QueryArgs,
        ServiceName as SailsServiceName,
        FunctionName,
        QueryName,
        QueryReturn,
        EventReturn`,
      )
      .import(`./${LIB_FILE_NAME}`, 'Program');
  };

  private generateTypes = () => {
    this._out
      .line("type UseProgramParameters = Omit<useSailsProgramParameters<Program>, 'library'>")
      .line('type ProgramType = InstanceType<typeof Program>')
      .line('type ServiceName = SailsServiceName<ProgramType>')
      .line('type ProgramParameter = { program: ProgramType | undefined }')
      .line()
      .line('type UseQueryParameters<', false)
      .increaseIndent()
      .line('TServiceName extends ServiceName,', false)
      .line('TFunctionName extends QueryName<ProgramType[TServiceName]>,', false)
      .reduceIndent()
      .line('> = Omit<', false)
      .increaseIndent()
      .line('UseProgramQueryParameters<', false)
      .increaseIndent()
      .line('ProgramType,', false)
      .line('TServiceName,', false)
      .line('TFunctionName,', false)
      .line('QueryArgs<ProgramType[TServiceName][TFunctionName]>,', false)
      .line('QueryReturn<ProgramType[TServiceName][TFunctionName]>', false)
      .reduceIndent()
      .line('>,', false)
      .line("'program' | 'serviceName' | 'functionName'", false)
      .reduceIndent()
      .line('> & ProgramParameter')
      .line()
      .line('type UseEventParameters<', false)
      .increaseIndent()
      .line('TServiceName extends ServiceName,', false)
      .line('TFunctionName extends FunctionName<ProgramType[TServiceName], EventReturn>,', false)
      .reduceIndent()
      .line('> = Omit<', false)
      .increaseIndent()
      .line('UseProgramEventParameters<', false)
      .increaseIndent()
      .line('ProgramType,', false)
      .line('TServiceName,', false)
      .line('TFunctionName,', false)
      .line('EventCallbackArgs<Event<ProgramType[TServiceName][TFunctionName]>>', false)
      .reduceIndent()
      .line('>,', false)
      .line("'program' | 'serviceName' | 'functionName'", false)
      .reduceIndent()
      .line('> & ProgramParameter')
      .line();
  };

  private generateUseProgram = () =>
    this._out
      .block('export function useProgram(parameters: UseProgramParameters)', () =>
        this._out.line('return useSailsProgram({ library: Program, ...parameters })'),
      )
      .line();

  private generateUseSendTransaction = (serviceName: string, functionName: string) => {
    const name = `useSend${serviceName}${functionName}Transaction`;

    this._out
      .block(`export function ${name}({ program }: ProgramParameter)`, () =>
        this._out.line(
          `return useSendProgramTransaction({ program, serviceName: '${toLowerCaseFirst(
            serviceName,
          )}', functionName: '${toLowerCaseFirst(functionName)}' })`,
        ),
      )
      .line();
  };

  private generateUsePrepareTransaction = (serviceName: string, functionName: string) => {
    const name = `usePrepare${serviceName}${functionName}Transaction`;

    this._out
      .block(`export function ${name}({ program }: ProgramParameter)`, () =>
        this._out.line(
          `return usePrepareProgramTransaction({ program, serviceName: '${toLowerCaseFirst(
            serviceName,
          )}', functionName: '${toLowerCaseFirst(functionName)}' })`,
        ),
      )
      .line();
  };

  private generateUseQuery = (serviceName: string, functionName: string) => {
    const name = `use${serviceName}${functionName}Query`;
    const formattedServiceName = toLowerCaseFirst(serviceName);
    const formattedFunctionName = toLowerCaseFirst(functionName);

    this._out
      .block(
        `export function ${name}(parameters: UseQueryParameters<'${formattedServiceName}', '${formattedFunctionName}'>)`,
        () =>
          this._out.line(
            `return useProgramQuery({ ...parameters, serviceName: '${formattedServiceName}', functionName: '${formattedFunctionName}' })`,
          ),
      )
      .line();
  };

  private generateUseEvent = (serviceName: string, eventName: string) => {
    const name = `use${serviceName}${eventName}Event`;
    const formattedServiceName = toLowerCaseFirst(serviceName);
    const functionName = `subscribeTo${eventName}Event`;

    this._out
      .block(
        `export function ${name}(parameters: UseEventParameters<'${formattedServiceName}', '${functionName}'>)`,
        () =>
          this._out.line(
            `return useProgramEvent({...parameters, serviceName: '${formattedServiceName}', functionName: '${functionName}' })`,
          ),
      )
      .line();
  };

  public generate() {
    const { services } = this._program;

    this.generateImports();
    this.generateTypes();
    this.generateUseProgram();

    Object.values(services).forEach(({ funcs, events, ...service }) => {
      funcs.forEach(({ isQuery, name }) => {
        if (isQuery) return this.generateUseQuery(service.name, name);

        this.generateUseSendTransaction(service.name, name);
        this.generateUsePrepareTransaction(service.name, name);
      });

      events.forEach(({ name }) => this.generateUseEvent(service.name, name));
    });
  }
}

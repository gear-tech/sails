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
      .import('@gear-js/api', 'HexString')
      .import(
        '@gear-js/react-hooks',
        'useProgram as useSailsProgram, useSendProgramTransaction, useProgramQuery, useProgramEvent, UseProgramParameters as useSailsProgramParameters, UseProgramQueryParameters, UseProgramEventParameters',
      )
      .import('react', 'createContext, ReactNode, useContext')
      // TODO: combine with above after hooks update
      .import(
        '@gear-js/react-hooks/dist/esm/hooks/sails/types',
        'Event, EventCallbackArgs, QueryArgs, ServiceName as SailsServiceName, FunctionName, QueryName, QueryReturn, EventReturn',
      )
      .import(`./${LIB_FILE_NAME}`, 'Program');
  };

  private generateTypes = () => {
    this._out
      .line("type UseProgramParameters = Omit<useSailsProgramParameters<Program>, 'library'>")
      .line('type ProgramType = InstanceType<typeof Program>')
      .line('type ServiceName = SailsServiceName<ProgramType>')
      .line('type ProgramId = { programId?: HexString | undefined }')
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
      .line('> & ProgramId')
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
      .line('> & ProgramId')
      .line();
  };

  private generateProgramIdContext = () => {
    this._out
      .line('const ProgramIdContext = createContext<HexString | undefined>(undefined)')
      .line('const useProgramId = () => useContext(ProgramIdContext)')
      .line('const { Provider } = ProgramIdContext')
      .line()
      .block('type Props =', () => this._out.line('value: HexString | undefined').line('children: ReactNode'))
      .line()
      .block('export function ProgramIdProvider({ children, value }: Props)', () =>
        this._out.line('return <Provider value={value}>{children}</Provider>'),
      )
      .line();
  };

  private generateUseProgram = () =>
    this._out
      .block('export function useProgram(parameters?: UseProgramParameters)', () =>
        this._out
          .line('const contextId = useProgramId()')
          .line("const id = parameters && 'id' in parameters ? parameters.id : contextId")
          .line()
          .line('return useSailsProgram({ library: Program, id, ...parameters })'),
      )
      .line();

  private generateUseProgramCall = () =>
    this._out
      .line(
        "const { data: program } = useProgram(parameters && 'programId' in parameters ? { id: parameters.programId } : undefined)",
      )
      .line();

  private generateUseSendTransaction = (serviceName: string, functionName: string) => {
    const name = `useSend${serviceName}${functionName}Transaction`;

    this._out
      .block(`export function ${name}(parameters?: ProgramId)`, () =>
        this.generateUseProgramCall().line(
          `return useSendProgramTransaction({ program, serviceName: '${toLowerCaseFirst(
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
          this.generateUseProgramCall().line(
            `return useProgramQuery({ program, serviceName: '${formattedServiceName}', functionName: '${formattedFunctionName}', ...parameters })`,
          ),
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
        `export function ${name}(parameters: UseEventParameters<'${formattedServiceName}', '${functionName}'>)`,
        () =>
          this.generateUseProgramCall().line(
            `return useProgramEvent({ program, serviceName: '${formattedServiceName}', functionName: '${functionName}', ...parameters })`,
          ),
      )
      .line();
  };

  public generate() {
    const { services } = this._program;

    this.generateImports();
    this.generateTypes();
    this.generateProgramIdContext();
    this.generateUseProgram();

    Object.values(services).forEach(({ funcs, events, ...service }) => {
      funcs.forEach(({ isQuery, name }) =>
        (isQuery ? this.generateUseQuery : this.generateUseSendTransaction)(service.name, name),
      );

      events.forEach(({ name }) => this.generateUseEvent(service.name, name));
    });
  }
}

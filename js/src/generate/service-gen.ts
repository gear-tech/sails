import { getPayloadMethod, getJsTypeDef, getScaleCodecDef } from '../utils/index.js';
import { FuncParam, Program } from '../parser/visitor.js';
import { Output } from './output.js';

const HEX_STRING_TYPE = '`0x${string}`';

const getArgs = (params: FuncParam[]) => {
  return params.map(({ name, def }) => `${name}: ${getJsTypeDef(def)}`).join(', ');
};

const getAccount = (isQuery: boolean) => {
  return isQuery ? '' : `, account: ${HEX_STRING_TYPE} | IKeyringPair, signerOptions?: Partial<SignerOptions>`;
};

const getValue = (isQuery: boolean) => {
  return isQuery ? '' : `, value?: number | string | bigint`;
};

export class ServiceGenerator {
  constructor(private _out: Output, private _program: Program, private scaleTypes: Record<string, any>) {}

  public generate() {
    this._out
      .import('@gear-js/api', 'GearApi')
      .import('./transaction.js', 'Transaction')
      .block(`export class Program extends Transaction`, () => {
        this._out.block(`constructor(api: GearApi, public programId?: ${HEX_STRING_TYPE})`, () => {
          this._out
            .block(`const types: Record<string, any> =`, () => {
              for (const [name, type] of Object.entries(this.scaleTypes)) {
                this._out.line(`${name}: ${JSON.stringify(type)},`, false);
              }
            })
            .line('super(api, types)');
        });
        this.generateProgramConstructor();
        this.generateMethods();
      });
  }

  private generateProgramConstructor() {
    this._out
      .block(
        `public createProgram(code: Uint8Array | Buffer, account: string | IKeyringPair, signerOptions: Partial<SignerOptions>, value = 0)`,
        () => {
          this._out.line(`return this.uploadProgram(code, 'New', account, signerOptions, value)`);
        },
      )
      .line()
      .block(
        `static async new(api: GearApi, code: Uint8Array | Buffer, account: string | IKeyringPair, signerOptions?: Partial<SignerOptions>, value = 0)`,
        () => {
          this._out
            .line(`const program = new Program(api)`)
            .line(`const { programId, response } = await program.createProgram(code, account, signerOptions, value)`)
            .line('await response()')
            .line('program.programId = programId')
            .line('return program');
        },
      )
      .line();
  }

  private generateMethods() {
    this._out.import('@polkadot/types/types', 'IKeyringPair');
    this._out.import('@polkadot/api/types', 'SignerOptions');

    for (const { name, def, params, isQuery } of this._program.service.funcs) {
      const returnType = getJsTypeDef(def);
      const returnScaleType = getScaleCodecDef(def);

      this._out
        .line()
        .block(
          `public async ${name[0].toLowerCase() + name.slice(1)}(${getArgs(params)}${getAccount(isQuery)}${getValue(
            isQuery,
          )}): Promise<${isQuery ? returnType : `IMethodReturnType<${returnType}>`}>`,
          () => {
            if (params.length === 0) {
              this._out.line(`const payload = this.registry.createType('String', '${name}').toU8a()`);
            } else {
              this._out.line(
                `const payload = this.registry.createType('(String, ${params
                  .map(({ def }) => getScaleCodecDef(def))
                  .join(', ')})', ['${name}', ${params.map(({ name }) => name).join(', ')}]).toU8a()`,
              );
            }

            if (isQuery) {
              this._out
                .line(`const stateBytes = await this.api.programState.read({ programId: this.programId, payload })`)
                .line(`const result = this.registry.createType('(String, ${returnScaleType})', stateBytes)`)
                .line(`return result[1].${getPayloadMethod(returnScaleType)}() as unknown as ${returnType}`);
            } else {
              this._out.import('./transaction.js', 'IMethodReturnType');
              this._out.block(
                `return this.submitMsg<${returnType}>`,
                () => {
                  this._out
                    .line('this.programId,', false)
                    .line('payload,', false)
                    .line(`'(String, ${returnScaleType})',`, false)
                    .line('account,', false)
                    .line('signerOptions,', false)
                    .line('value,', false);
                },
                '(',
              );
            }
          },
        );
    }
  }
}

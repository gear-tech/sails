import { getPayloadMethod, getJsTypeDef, getScaleCodecDef } from '../utils/index.js';
import { FuncParam, Program } from '../parser/visitor.js';
import { Output } from './output.js';

const HEX_STRING_TYPE = '`0x${string}`';

const getArgs = (params: FuncParam[]) => {
  if (params.length === 0) {
    return null;
  }
  return params.map(({ name, def }) => `${name}: ${getJsTypeDef(def)}`).join(', ');
};

const getAccount = (isQuery: boolean) => {
  return isQuery ? '' : `, account: ${HEX_STRING_TYPE} | IKeyringPair, signerOptions?: Partial<SignerOptions>`;
};

const getValue = (isQuery: boolean) => {
  return isQuery ? '' : `, value?: number | string | bigint`;
};

const getFuncName = (name: string) => {
  return name[0].toLowerCase() + name.slice(1);
};

export class ServiceGenerator {
  constructor(private _out: Output, private _program: Program, private scaleTypes: Record<string, any>) {}

  public generate() {
    this._out
      .import('@gear-js/api', 'GearApi')
      .import('./transaction.js', 'Transaction')
      .block(`export class Program extends Transaction`, () => {
        this._out
          .block(`constructor(api: GearApi, public programId?: ${HEX_STRING_TYPE})`, () => {
            this._out
              .block(`const types: Record<string, any> =`, () => {
                for (const [name, type] of Object.entries(this.scaleTypes)) {
                  this._out.line(`${name}: ${JSON.stringify(type)},`, false);
                }
              })
              .line('super(api, types)');
          })
          .line();
        this.generateProgramConstructor();
        this.generateMethods();
        this.generateSubscriptions();
      });
  }

  private generateProgramConstructor() {
    for (const { name, params } of this._program.ctor.funcs) {
      const args = getArgs(params);
      this._out
        .block(
          `async ${getFuncName(name)}Ctor(code: Uint8Array | Buffer, ${
            args !== null ? args + ', ' : ''
          }account: string | IKeyringPair, signerOptions?: Partial<SignerOptions>, value = 0)`,
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
            this._out
              .line(
                `const { programId, response } = await this.uploadProgram(code, payload, account, signerOptions, value)`,
              )
              .line('await response()')
              .line('this.programId = programId')
              .line('return this');
          },
        )
        .line();
    }
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
          `public async ${getFuncName(name)}(${getArgs(params)}${getAccount(isQuery)}${getValue(isQuery)}): Promise<${
            isQuery ? returnType : `IMethodReturnType<${returnType}>`
          }>`,
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

  private generateSubscriptions() {
    if (this._program.service.events.length > 0) {
      this._out
        .firstLine(`const ZERO_ADDRESS = u8aToHex(new Uint8Array(32))`)
        .import('@polkadot/util', 'u8aToHex')
        .import('@polkadot/util', 'compactFromU8aLim');
    }

    for (const event of this._program.service.events) {
      const jsType = getJsTypeDef(event.def);

      this._out
        .line()
        .block(
          `public subscribeTo${event.name}Event(callback: (data: ${jsType}) => void | Promise<void>): Promise<() => void>`,
          () => {
            this._out
              .line(`return this.api.gearEvents.subscribeToGearEvent('UserMessageSent', ({ data: { message } }) => {`)
              .increaseIndent()
              .block(`if (!message.source.eq(this.programId) || !message.destination.eq(ZERO_ADDRESS))`, () => {
                this._out.line(`return`);
              })
              .line()
              .line(`const payload = message.payload.toU8a()`)
              .line(`const [offset, limit] = compactFromU8aLim(payload)`)
              .line(`const name = this.registry.createType('String', payload.subarray(offset, limit)).toString()`)
              .block(`if (name === '${event.name}')`, () => {
                this._out
                  .line(
                    `callback(this.registry.createType('(String, ${getScaleCodecDef(
                      event.def,
                      true,
                    )})', message.payload)[1].toJSON() as ${jsType})`,
                  )
              })
              .reduceIndent()
              .line(`})`);
          },
        );
    }
  }
}

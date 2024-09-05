import { getPayloadMethod, getScaleCodecDef, PayloadMethod, toLowerCaseFirst } from '../utils/index.js';
import { Output } from './output.js';
import { BaseGenerator } from './base.js';
import { ISailsFuncParam, ISailsProgram, ISailsService } from 'sails-js-types';

const HEX_STRING_TYPE = '`0x${string}`';

const VALUE_ARG = 'value?: number | string | bigint';

const getFuncName = (name: string) => {
  return name[0].toLowerCase() + name.slice(1);
};

const createPayload = (serviceName: string, fnName: string, params: ISailsFuncParam[]) => {
  if (params.length === 0) {
    return `const payload = this._program.registry.createType('(String, String)', ['${serviceName}', '${fnName}']).toHex()`;
  } else {
    return `const payload = this._program.registry.createType('(String, String, ${params
      .map(({ def }) => getScaleCodecDef(def))
      .join(', ')})', ['${serviceName}', '${fnName}', ${params.map(({ name }) => name).join(', ')}]).toHex()`;
  }
};

export class ServiceGenerator extends BaseGenerator {
  constructor(
    out: Output,
    private _program: ISailsProgram,
    private scaleTypes: Record<string, any>,
  ) {
    super(out);
  }

  public generate(className = 'Program') {
    this._out
      .import('@gear-js/api', 'GearApi')
      .import(`@polkadot/types`, `TypeRegistry`)
      .import('sails-js', 'TransactionBuilder')
      .block(`export class ${className}`, () => {
        this._out.line(`public readonly registry: TypeRegistry`);

        for (const service of this._program.services) {
          this._out.line(
            `public readonly ${toLowerCaseFirst(service.name)}: ${
              service.name === className ? service.name + 'Service' : service.name
            }`,
          );
        }

        this._out
          .line()
          .block(`constructor(public api: GearApi, public programId?: ${HEX_STRING_TYPE})`, () => {
            this._out
              .block(`const types: Record<string, any> =`, () => {
                for (const [name, type] of Object.entries(this.scaleTypes)) {
                  this._out.line(`${name}: ${JSON.stringify(type)},`, false);
                }
              })
              .line()
              .line(`this.registry = new TypeRegistry()`)
              .line(`this.registry.setKnownTypes({ types })`)
              .line(`this.registry.register(types)`)
              .line();

            for (const service of this._program.services) {
              this._out.line(`this.${toLowerCaseFirst(service.name)} = new ${service.name}(this)`);
            }
          })
          .line();
        this.generateProgramConstructor();
      });
    this.generateServices(className);
  }

  private generateProgramConstructor() {
    if (!this._program.ctor || this._program.ctor.funcs.length === 0) return;

    for (const { name, params } of this._program.ctor.funcs) {
      const args = this.getArgs(params);
      this._out
        .block(
          `${getFuncName(name)}CtorFromCode(code: Uint8Array | Buffer${
            args !== null ? ', ' + args : ''
          }): TransactionBuilder<null>`,
          () => {
            this._out
              .line(`const builder = new TransactionBuilder<null>(`, false)
              .increaseIndent()
              .line(`this.api,`, false)
              .line(`this.registry,`, false)
              .line(`'upload_program',`, false)
              .line(
                params.length === 0 ? `'${name}',` : `['${name}', ${params.map(({ name }) => name).join(', ')}],`,
                false,
              )
              .line(
                params.length === 0
                  ? `'String',`
                  : `'(String, ${params.map(({ def }) => getScaleCodecDef(def)).join(', ')})',`,
                false,
              )
              .line(`'String',`, false)
              .line(`code,`, false)
              .reduceIndent()
              .line(`)`)
              .line()
              .line('this.programId = builder.programId')
              .line('return builder');
          },
        )
        .line()
        .block(
          `${getFuncName(name)}CtorFromCodeId(codeId: ${HEX_STRING_TYPE}${args !== null ? ', ' + args : ''})`,
          () => {
            this._out
              .line(`const builder = new TransactionBuilder<null>(`, false)
              .increaseIndent()
              .line(`this.api,`, false)
              .line(`this.registry,`, false)
              .line(`'create_program',`, false)
              .line(
                params.length === 0 ? `'${name}',` : `['${name}', ${params.map(({ name }) => name).join(', ')}],`,
                false,
              )
              .line(
                params.length === 0
                  ? `'String',`
                  : `'(String, ${params.map(({ def }) => getScaleCodecDef(def)).join(', ')})',`,
                false,
              )
              .line(`'String',`, false)
              .line(`codeId,`, false)
              .reduceIndent()
              .line(`)`)
              .line()
              .line('this.programId = builder.programId')
              .line('return builder');
          },
        );
    }
  }

  private generateServices(programClass: string) {
    for (const service of this._program.services) {
      this._out
        .line()
        .block(`export class ${service.name === programClass ? service.name + 'Service' : service.name}`, () => {
          this._out.line(`constructor(private _program: ${programClass}) {}`, false);
          this.generateMethods(service);
          this.generateSubscriptions(service);
        });
    }
  }

  private generateMethods(service: ISailsService) {
    for (const { name, def, params, isQuery } of service.funcs) {
      const returnScaleType = getScaleCodecDef(def);
      const decodeMethod = getPayloadMethod(returnScaleType);
      const returnType = this.getType(def, decodeMethod);

      this._out.line().block(this.getFuncSignature(name, params, returnType, isQuery), () => {
        if (isQuery) {
          this._out
            .line(createPayload(service.name, name, params))
            .import('@gear-js/api', 'decodeAddress')
            .line(`const reply = await this._program.api.message.calculateReply({`, false)
            .increaseIndent()
            .line(`destination: this._program.programId,`, false)
            .line(`origin: originAddress ? decodeAddress(originAddress) : ZERO_ADDRESS,`, false)
            .line(`payload,`, false)
            .line(`value: value || 0,`, false)
            .line(`gasLimit: this._program.api.blockGasLimit.toBigInt(),`, false)
            .line(`at: atBlock || null,`, false)
            .reduceIndent()
            .line(`})`)
            .line(
              "if (!reply.code.isSuccess) throw new Error(this._program.registry.createType('String', reply.payload).toString())",
            )
            .line(
              `const result = this._program.registry.createType('(String, String, ${returnScaleType})', reply.payload)`,
            )
            .line(`return result[2].${decodeMethod}() as unknown as ${returnType}`);
        } else {
          this._out
            .line(`if (!this._program.programId) throw new Error('Program ID is not set')`)
            .line(`return new TransactionBuilder<${returnType}>(`, false)
            .increaseIndent()
            .line(`this._program.api,`, false)
            .line(`this._program.registry,`, false)
            .line(`'send_message',`, false)
            .line(
              params.length === 0
                ? `['${service.name}', '${name}'],`
                : `['${service.name}', '${name}', ${params.map(({ name }) => name).join(', ')}],`,
              false,
            )
            .line(
              params.length === 0
                ? `'(String, String)',`
                : `'(String, String, ${params.map(({ def }) => getScaleCodecDef(def)).join(', ')})',`,
              false,
            )
            .line(`'${returnScaleType}',`, false)
            .line(`this._program.programId`, false)
            .reduceIndent()
            .line(`)`);
        }
      });
    }
  }

  private generateSubscriptions(service: ISailsService) {
    if (service.events.length > 0) {
      this._out
        .import('sails-js', 'getServiceNamePrefix')
        .import('sails-js', 'getFnNamePrefix')
        .import('sails-js', 'ZERO_ADDRESS');
    }

    for (const event of service.events) {
      const decodeMethod = event.def ? getPayloadMethod(getScaleCodecDef(event.def)) : PayloadMethod.toJSON;
      const jsType = event.def ? this.getType(event.def, decodeMethod) : 'null';

      this._out
        .line()
        .block(
          `public subscribeTo${event.name}Event(callback: (data: ${jsType}) => void | Promise<void>): Promise<() => void>`,
          () => {
            this._out
              .line(
                `return this._program.api.gearEvents.subscribeToGearEvent('UserMessageSent', ({ data: { message } }) => {`,
              )
              .increaseIndent()
              .block(
                `if (!message.source.eq(this._program.programId) || !message.destination.eq(ZERO_ADDRESS))`,
                () => {
                  this._out.line(`return`);
                },
              )
              .line()
              .line(`const payload = message.payload.toHex()`)
              .block(
                `if (getServiceNamePrefix(payload) === '${service.name}' && getFnNamePrefix(payload) === '${event.name}')`,
                () => {
                  if (jsType === 'null') {
                    this._out.line(`callback(null)`);
                  } else {
                    this._out.line(
                      `callback(this._program.registry.createType('(String, String, ${getScaleCodecDef(
                        event.def,
                        true,
                      )})', message.payload)[2].${decodeMethod}() as unknown as ${jsType})`,
                    );
                  }
                },
              )
              .reduceIndent()
              .line(`})`);
          },
        );
    }
  }

  private getArgs(params: ISailsFuncParam[]) {
    if (params.length === 0) {
      return null;
    }
    return params.map(({ name, def }) => `${name}: ${this.getType(def)}`).join(', ');
  }

  private getFuncSignature(name: string, params: ISailsFuncParam[], returnType: string, isQuery: boolean) {
    const args = this.getArgs(params);

    let result = `public ${isQuery ? 'async ' : ''}${getFuncName(name)}(${args || ''}`;

    if (isQuery) {
      result += `${args ? ', ' : ''}originAddress?: string, ${VALUE_ARG}, atBlock?: ${HEX_STRING_TYPE}`;
    }

    result += `): ${isQuery ? `Promise<${returnType}>` : `TransactionBuilder<${returnType}>`}`;

    return result;
  }
}

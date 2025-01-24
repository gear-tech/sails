import { getPayloadMethod, getScaleCodecDef, PayloadMethod, toLowerCaseFirst } from 'sails-js-util';
import { ISailsFuncParam, ISailsProgram, ISailsService } from 'sails-js-types';
import { Output } from './output.js';
import { BaseGenerator } from './base.js';
import { formatDocs } from './format.js';

const HEX_STRING_TYPE = '`0x${string}`';

const VALUE_ARG = 'value?: number | string | bigint';

const getFuncName = (name: string) => {
  return name[0].toLowerCase() + name.slice(1);
};

const createPayload = (serviceName: string, fnName: string, params: ISailsFuncParam[]) => {
  return params.length === 0
    ? `const payload = this._program.registry.createType('(String, String)', ['${serviceName}', '${fnName}']).toHex()`
    : `const payload = this._program.registry.createType('(String, String, ${params
        .map(({ def }) => getScaleCodecDef(def))
        .join(', ')})', ['${serviceName}', '${fnName}', ${params.map(({ name }) => name).join(', ')}]).toHex()`;
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
    const $ = this._out;
    const _classNameTitled = className[0].toUpperCase() + className.slice(1);

    $.import('@gear-js/api', 'GearApi')
      .import(`@polkadot/types`, `TypeRegistry`)
      .import('sails-js', 'TransactionBuilder')
      .block(`export class ${_classNameTitled}`, () => {
        $.line(`public readonly registry: TypeRegistry`);

        for (const service of this._program.services) {
          $.line(
            `public readonly ${toLowerCaseFirst(service.name)}: ${
              service.name === _classNameTitled ? service.name + 'Service' : service.name
            }`,
          );
        }

        $.line()
          .block(`constructor(public api: GearApi, private _programId?: ${HEX_STRING_TYPE})`, () => {
            $.block(`const types: Record<string, any> =`, () => {
              for (const [name, type] of Object.entries(this.scaleTypes)) {
                $.line(`${name}: ${JSON.stringify(type)},`, false);
              }
            })
              .line()
              .line(`this.registry = new TypeRegistry()`)
              .line(`this.registry.setKnownTypes({ types })`)
              .line(`this.registry.register(types)`)
              .line();

            for (const service of this._program.services) {
              $.line(`this.${toLowerCaseFirst(service.name)} = new ${service.name}(this)`);
            }
          })
          .line()
          .block(`public get programId(): ${HEX_STRING_TYPE}`, () => {
            $.line('if (!this._programId) throw new Error(`Program ID is not set`)').line('return this._programId');
          })
          .line();
        this.generateProgramConstructor();
      });
    this.generateServices(_classNameTitled);
  }

  private generateProgramConstructor() {
    if (!this._program.ctor || this._program.ctor.funcs.length === 0) return;

    const $ = this._out;

    for (const { name, params, docs } of this._program.ctor.funcs) {
      const args = this.getArgs(params);

      const ctorDocs = formatDocs(docs);

      $.lines(ctorDocs, false)
        .block(
          `${getFuncName(name)}CtorFromCode(code: Uint8Array | Buffer${
            args === null ? '' : ', ' + args
          }): TransactionBuilder<null>`,
          () => {
            $.line(`const builder = new TransactionBuilder<null>(`, false)
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
              .line('this._programId = builder.programId')
              .line('return builder');
          },
        )
        .line()
        .lines(ctorDocs, false)
        .block(
          `${getFuncName(name)}CtorFromCodeId(codeId: ${HEX_STRING_TYPE}${args === null ? '' : ', ' + args})`,
          () => {
            $.line(`const builder = new TransactionBuilder<null>(`, false)
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
              .line('this._programId = builder.programId')
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
    for (const { name, def, params, isQuery, docs } of service.funcs) {
      const returnScaleType = getScaleCodecDef(def);
      const decodeMethod = getPayloadMethod(returnScaleType);
      const returnType = this.getType(def, decodeMethod);

      this._out
        .line()
        .lines(formatDocs(docs), false)
        .block(this.getFuncSignature(name, params, returnType, isQuery), () => {
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
              .line(`at: atBlock,`, false)
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
    const $ = this._out;
    if (service.events.length > 0) {
      $.import('sails-js', 'getServiceNamePrefix')
        .import('sails-js', 'getFnNamePrefix')
        .import('sails-js', 'ZERO_ADDRESS');
    }

    for (const event of service.events) {
      const decodeMethod = event.def ? getPayloadMethod(getScaleCodecDef(event.def)) : PayloadMethod.toJSON;
      const jsType = event.def ? this.getType(event.def, decodeMethod) : 'null';

      $.line()
        .lines(formatDocs(event.docs), false)
        .block(
          `public subscribeTo${event.name}Event(callback: (data: ${jsType}) => void | Promise<void>): Promise<() => void>`,
          () => {
            $.line(
              `return this._program.api.gearEvents.subscribeToGearEvent('UserMessageSent', ({ data: { message } }) => {`,
            )
              .increaseIndent()
              .block(
                `if (!message.source.eq(this._program.programId) || !message.destination.eq(ZERO_ADDRESS))`,
                () => {
                  $.line(`return`);
                },
              )
              .line()
              .line(`const payload = message.payload.toHex()`)
              .block(
                `if (getServiceNamePrefix(payload) === '${service.name}' && getFnNamePrefix(payload) === '${event.name}')`,
                () => {
                  if (jsType === 'null') {
                    $.line(`callback(null)`);
                  } else {
                    $.line(
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

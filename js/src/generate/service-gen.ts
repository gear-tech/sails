import { getPayloadMethod, getJsTypeDef, getScaleCodecDef } from '../utils/index.js';
import { FuncParam, Program } from '../parser/visitor.js';
import { Output } from './output.js';

const HEX_STRING_TYPE = '`0x${string}`';

const VALUE_ARG = 'value?: number | string | bigint';

const getArgs = (params: FuncParam[]) => {
  if (params.length === 0) {
    return null;
  }
  return params.map(({ name, def }) => `${name}: ${getJsTypeDef(def)}`).join(', ');
};

const getFuncName = (name: string) => {
  return name[0].toLowerCase() + name.slice(1);
};

const createPayload = (name: string, params: FuncParam[]) => {
  if (params.length === 0) {
    return `const payload = this.registry.createType('String', '${name}').toU8a()`;
  } else {
    return `const payload = this.registry.createType('(String, ${params
      .map(({ def }) => getScaleCodecDef(def))
      .join(', ')})', ['${name}', ${params.map(({ name }) => name).join(', ')}]).toU8a()`;
  }
};

const getFuncSignature = (name: string, params: FuncParam[], returnType: string, isQuery: boolean) => {
  const args = getArgs(params);

  let result = `public ${isQuery ? 'async ' : ''}${getFuncName(name)}(${args || ''}`;

  if (isQuery) {
    result += `${args ? ', ' : ''}originAddress: string, ${VALUE_ARG}, atBlock?: ${HEX_STRING_TYPE}`;
  }

  result += `): ${isQuery ? `Promise<${returnType}>` : `TransactionBuilder<${returnType}>`}`;

  return result;
};

export class ServiceGenerator {
  constructor(private _out: Output, private _program: Program, private scaleTypes: Record<string, any>) {}

  public generate(className = 'Program') {
    this._out
      .import('@gear-js/api', 'GearApi')
      .import(`@polkadot/types`, `TypeRegistry`)
      .import('sails-js', 'TransactionBuilder')
      .block(`export class ${className}`, () => {
        this._out
          .line(`private registry: TypeRegistry`)
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
              .line(`this.registry.register(types)`);
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

  private generateMethods() {
    for (const { name, def, params, isQuery } of this._program.service.funcs) {
      const returnType = getJsTypeDef(def);
      const returnScaleType = getScaleCodecDef(def);

      this._out.line().block(getFuncSignature(name, params, returnType, isQuery), () => {
        if (isQuery) {
          this._out
            .line(createPayload(name, params))
            .import('@gear-js/api', 'decodeAddress')
            .line(`const reply = await this.api.message.calculateReply({`, false)
            .increaseIndent()
            .line(`destination: this.programId,`, false)
            .line(`origin: decodeAddress(originAddress),`, false)
            .line(`payload,`, false)
            .line(`value: value || 0,`, false)
            .line(`gasLimit: this.api.blockGasLimit.toBigInt(),`, false)
            .line(`at: atBlock || null,`, false)
            .reduceIndent()
            .line(`})`)
            .line(`const result = this.registry.createType('(String, ${returnScaleType})', reply.payload)`)
            .line(`return result[1].${getPayloadMethod(returnScaleType)}() as unknown as ${returnType}`);
        } else {
          this._out
            .line(`return new TransactionBuilder<${returnType}>(`, false)
            .increaseIndent()
            .line(`this.api,`, false)
            .line(`this.registry,`, false)
            .line(`'send_message',`, false)
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
            .line(`'${returnScaleType}',`, false)
            .line(`this.programId`, false)
            .reduceIndent()
            .line(`)`);
        }
      });
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
                this._out.line(
                  `callback(this.registry.createType('(String, ${getScaleCodecDef(
                    event.def,
                    true,
                  )})', message.payload)[1].toJSON() as ${jsType})`,
                );
              })
              .reduceIndent()
              .line(`})`);
          },
        );
    }
  }
}

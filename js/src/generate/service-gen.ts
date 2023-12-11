import { IArgument, IService, IServiceMethod, MethodType } from '../types/index.js';
import { getClassName } from './utils/index.js';
import { getType } from './utils/scale-types.js';
import { getTypeDef } from './utils/types.js';
import { Output } from './output.js';

const HEX_STRING_TYPE = '`0x${string}`';

const getArgs = (args: IArgument[]) => {
  return (
    args.map(({ name, type }) => `${name}: ${getTypeDef(type, false, true)}`).join(', ') + (args.length > 0 ? ', ' : '')
  );
};

const getAccount = (kind: MethodType) => {
  return kind === 'message' ? `account: ${HEX_STRING_TYPE} | IKeyringPair` : '';
};

export class ServiceGenerator {
  private scaleTypes: Record<string, any> = {};
  constructor(private _out: Output) {}

  public generate(service: IService, scaleTypes: Record<string, any>) {
    this.scaleTypes = scaleTypes;
    this._out
      .import('@gear-js/api', 'GearApi')
      .import('./transaction.js', 'Transaction')
      .block(`export class ${getClassName(service.name)} extends Transaction`, () => {
        this._out.block(`constructor(api: GearApi, public programId: ${HEX_STRING_TYPE})`, () => {
          this._out
            .block(`const types: Record<string, any> =`, () => {
              for (const [name, type] of Object.entries(this.scaleTypes)) {
                this._out.line(`${name}: ${JSON.stringify(type)},`, false);
              }
            })
            .line('super(api, types)');
        });
        this.generateMethods(service.methods);
      });
  }

  private generateMethods(methods: IServiceMethod[]) {
    this._out.import('@polkadot/types/types', 'IKeyringPair');

    for (const {
      def: { args, name, output },
      kind,
    } of methods) {
      const returnType = getTypeDef(output, false, true);

      this._out
        .line()
        .block(
          `public async ${name[0].toLowerCase() + name.slice(1)}(${getArgs(args)}${getAccount(
            kind,
          )}): Promise<${returnType}>`,
          () => {
            if (args.length === 0) {
              this._out.line(`const payload = this.registry.createType('String', '${name}/').toU8a()`);
            } else {
              this._out
                .line(`const payload = [`, false)
                .increaseIndent()
                .line(`...this.registry.createType('String', '${name}/').toU8a(),`, false);
              for (const { name, type } of args) {
                this._out.line(`...this.registry.createType('${getType(type, true)}', ${name}).toU8a(),`, false);
              }
              this._out.reduceIndent().line(']');
            }

            if (kind === 'message') {
              this._out
                .line(`const replyPayloadBytes = await this.submitMsgAndWaitForReply(`, false)
                .increaseIndent()
                .line('this.programId,', false)
                .line('payload,', false)
                .line('account,', false)
                .reduceIndent()
                .line(')')
                .line(`const result = this.registry.createType('${getType(output, true)}', replyPayloadBytes)`)
                .line(`return result.toJSON() as ${returnType}`);
            } else if (kind === 'query') {
              this._out
                .line(`const stateBytes = await this.api.programState.read({ programId: this.programId, payload})`)
                .line(`const result = this.registry.createType('${getType(output, true)}', stateBytes)`)
                .line(`return result.toJSON() as ${returnType}`);
            }
          },
        );
    }
  }
}

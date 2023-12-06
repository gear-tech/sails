import { CstNode } from 'chevrotain';

import { SailsParser } from './parser.js';
import {
  IArgument,
  IInnerType,
  IOptionType,
  IResultType,
  IService,
  IServiceMethod,
  IServiceMethodDef,
  IStructFieldDef,
  IStructType,
  ITupleType,
  IType,
  ITypeDecl,
  ITypeNameType,
  IVariantField,
  IVariantType,
  IVecType,
} from '../types/index.js';

export function getSailsVisitorClass(parser: SailsParser) {
  const BaseCstVisitor = parser.getBaseCstVisitorConstructor<any, any>();

  return class SailsVisitor extends BaseCstVisitor {
    constructor() {
      super();
      this.validateVisitor();
    }

    sails(ctx: CstNode & Record<string, any>) {
      const types: IType[] = [];

      if (ctx.type) {
        for (const type of ctx.type) {
          types.push(this.visit(type));
        }
      }

      const services = [];

      if (ctx.service) {
        for (const service of ctx.service) {
          services.push(this.visit(service));
        }
      }

      return {
        types,
        services,
      };
    }

    type(ctx: CstNode & Record<string, any>): IType {
      const type = this.visit(ctx.declaration);

      return { ...this.visit(ctx.def), type };
    }

    declaration(ctx: CstNode & Record<string, any>): ITypeDecl {
      const name = ctx.Identifier[0].image;
      if (ctx.generic) {
        return { name, kind: 'generic', generic: this.visit(ctx.generic) };
      }
      return { name, kind: 'simple' };
    }

    generic(ctx: CstNode & Record<string, any>): IInnerType[] {
      const genericInner = [];

      for (const type of ctx.def) {
        genericInner.push(this.visit(type));
      }

      return genericInner;
    }

    struct(ctx: CstNode & Record<string, any>): Omit<IStructType | ITupleType, 'type'> {
      if (ctx.structField) {
        const fields = ctx.structField.map((field) => this.visit(field));
        return { def: { fields }, kind: 'struct' };
      } else if (ctx.tupleField) {
        const fields = ctx.tupleField.map((field) => this.visit(field));
        return { def: { fields }, kind: 'tuple' };
      } else {
        throw new Error('Invalid struct definition');
      }
    }

    opt(ctx: CstNode & Record<string, any>): Omit<IOptionType, 'type'> {
      const type = this.visit(ctx.fieldType);
      return { def: type, kind: 'option' };
    }

    vec(ctx: CstNode & Record<string, any>): Omit<IVecType, 'type'> {
      const type = this.visit(ctx.fieldType);
      return { def: type, kind: 'vec' };
    }

    result(ctx: CstNode & Record<string, any>): Omit<IResultType, 'type'> {
      const ok = this.visit(ctx.fieldType[0]);
      const err = this.visit(ctx.fieldType[1]);
      return { def: { ok, err }, kind: 'result' };
    }

    structField(ctx: CstNode & Record<string, any>): IStructFieldDef {
      const name = this.visit(ctx.fieldName);
      const type = this.visit(ctx.fieldType);
      return { name, type };
    }

    tupleField(ctx: CstNode & Record<string, any>): Omit<ITupleType, 'type'> {
      return this.visit(ctx.fieldType);
    }

    fieldName(ctx: CstNode & Record<string, any>): string {
      return ctx.Identifier[0].image;
    }

    fieldType(ctx: CstNode & Record<string, any>): IInnerType {
      return this.visit(ctx.def);
    }

    def(ctx: CstNode & Record<string, any>): IInnerType {
      if (ctx.typeName) {
        return this.visit(ctx.typeName);
      }
      if (ctx.struct) {
        return this.visit(ctx.struct);
      }
      if (ctx.variant) {
        return this.visit(ctx.variant);
      }
      if (ctx.opt) {
        return this.visit(ctx.opt);
      }
      if (ctx.result) {
        return this.visit(ctx.result);
      }
      if (ctx.vec) {
        return this.visit(ctx.vec);
      }
    }

    typeName(ctx: CstNode & Record<string, any>): Omit<ITypeNameType, 'type'> {
      let def: ITypeDecl;

      const identifier = { name: ctx.Identifier[0].image };

      if (ctx.generic) {
        def = { ...identifier, kind: 'generic', generic: this.visit(ctx.generic) };
      } else {
        def = { ...identifier, kind: 'simple' };
      }
      return { def, kind: 'typeName' };
    }

    variant(ctx: CstNode & Record<string, any>): Omit<IVariantType, 'type'> {
      return { def: { variants: ctx.variantField.map((field) => this.visit(field)) }, kind: 'variant' };
    }

    variantField(ctx: CstNode & Record<string, any>): IVariantField {
      const name = this.visit(ctx.fieldName);
      if (ctx.fieldType) {
        const type = this.visit(ctx.fieldType);
        return { name, type };
      }
      return { name };
    }

    service(ctx: CstNode & Record<string, any>): IService {
      const methods: IServiceMethod[] = [];

      if (ctx.message) {
        for (const message of ctx.message) {
          methods.push({ def: this.visit(message), kind: 'message' });
        }
      }

      if (ctx.query) {
        for (const query of ctx.query) {
          methods.push({ def: this.visit(query), kind: 'query' });
        }
      }
      return { methods };
    }

    message(ctx: CstNode & Record<string, any>): IServiceMethodDef {
      return {
        name: this.visit(ctx.methodName),
        args: this.visit(ctx.methodArguments),
        output: this.visit(ctx.methodOutput),
      };
    }

    query(ctx: CstNode & Record<string, any>): IServiceMethodDef {
      return {
        name: this.visit(ctx.methodName),
        args: this.visit(ctx.methodArguments),
        output: this.visit(ctx.methodOutput),
      };
    }

    methodName(ctx: CstNode & Record<string, any>): string {
      return ctx.Identifier[0].image;
    }

    methodArguments(ctx: CstNode & Record<string, any>): IArgument[] {
      const args = [];
      if (!ctx.argument) {
        return args;
      }
      for (const arg of ctx.argument) {
        args.push(this.visit(arg));
      }
      return args;
    }

    argument(ctx: CstNode & Record<string, any>): IArgument {
      const name = this.visit(ctx.argumentName);
      const type = this.visit(ctx.argumentType);
      return { name, type };
    }

    argumentName(ctx: CstNode & Record<string, any>): string {
      return ctx.Identifier[0].image;
    }

    argumentType(ctx: CstNode & Record<string, any>): IInnerType {
      return this.visit(ctx.def);
    }

    methodOutput(ctx: CstNode & Record<string, any>): IInnerType {
      return this.visit(ctx.def);
    }
  };
}

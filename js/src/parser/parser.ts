import { CstNode, CstParser } from 'chevrotain';

import {
  ALL_TOKENS,
  Arrow,
  Colon,
  Comma,
  Equals,
  Identifier,
  LAngle,
  LCurly,
  LParen,
  Message,
  Query,
  RAngle,
  RCurly,
  RParen,
  Semicolon,
  Service,
  Struct,
  Option,
  Vec,
  Result,
  Type,
  Variant,
} from './tokens.js';

export class SailsParser extends CstParser {
  constructor() {
    super(ALL_TOKENS, { recoveryEnabled: true });

    const $ = this as SailsParser & Record<string, any>;

    this.commonParser();
    this.structParser();
    this.tupleParser();
    this.variantParser();
    this.optionParser();
    this.resultParser();
    this.vecParser();
    this.typesParser();
    this.servicesParser();
    this.methodParser();

    $.RULE('sails', () => {
      $.MANY({
        DEF: () => {
          $.OR([{ ALT: () => $.SUBRULE($.type) }, { ALT: () => $.SUBRULE($.service) }]);
        },
      });
    });

    this.performSelfAnalysis();
  }

  parse(): CstNode {
    return (this as any).sails();
  }

  private commonParser() {
    const $ = this as SailsParser & Record<string, any>;

    $.RULE('declaration', () => {
      $.CONSUME(Identifier);
      $.OPTION(() => $.SUBRULE($.generic));
    });

    $.RULE('typeName', () => {
      $.CONSUME(Identifier);
      $.OPTION(() => $.SUBRULE($.generic));
    });

    $.RULE('generic', () => {
      $.CONSUME(LAngle);
      $.SUBRULE($.def);
      $.OPTION(() => {
        $.MANY(() => {
          $.CONSUME(Comma);
          $.SUBRULE1($.def);
        });
      });
      $.CONSUME(RAngle);
    });

    $.RULE('def', () =>
      $.OR([
        { ALT: () => $.SUBRULE($.typeName) },
        { ALT: () => $.SUBRULE($.opt) },
        { ALT: () => $.SUBRULE($.vec) },
        { ALT: () => $.SUBRULE($.result) },
        { ALT: () => $.SUBRULE($.struct) },
        { ALT: () => $.SUBRULE($.variant) },
      ]),
    );

    $.RULE('fieldName', () => $.CONSUME(Identifier));
    $.RULE('fieldType', () => $.SUBRULE($.def));
  }

  private typesParser() {
    const $ = this as SailsParser & Record<string, any>;

    $.RULE('type', () => {
      $.CONSUME(Type);
      $.SUBRULE($.declaration);
      $.CONSUME(Equals);
      $.SUBRULE($.def);
      $.OPTION(() => $.CONSUME(Semicolon));
    });
  }

  private structParser() {
    const $ = this as SailsParser & Record<string, any>;

    $.RULE('struct', () => {
      $.CONSUME(Struct);
      $.CONSUME(LCurly);
      $.AT_LEAST_ONE({
        DEF: () => $.OR([{ ALT: () => $.SUBRULE($.structField) }, { ALT: () => $.SUBRULE($.tupleField) }]),
      });
      $.CONSUME(RCurly);
    });

    $.RULE('structField', () => {
      $.SUBRULE($.fieldName);
      $.CONSUME(Colon);
      $.SUBRULE($.fieldType);
      $.OPTION(() => $.CONSUME(Comma));
    });
  }

  private tupleParser() {
    const $ = this as SailsParser & Record<string, any>;

    $.RULE('tupleField', () => {
      $.SUBRULE($.fieldType);
      $.OPTION(() => $.CONSUME(Comma));
    });
  }

  private optionParser() {
    const $ = this as SailsParser & Record<string, any>;

    $.RULE('opt', () => {
      $.CONSUME(Option);
      $.SUBRULE($.fieldType);
    });
  }

  private vecParser() {
    const $ = this as SailsParser & Record<string, any>;

    $.RULE('vec', () => {
      $.CONSUME(Vec);
      $.SUBRULE($.fieldType);
    });
  }

  private resultParser() {
    const $ = this as SailsParser & Record<string, any>;

    $.RULE('result', () => {
      $.CONSUME(Result);
      $.CONSUME(LParen);
      $.SUBRULE($.fieldType);
      $.CONSUME(Comma);
      $.SUBRULE1($.fieldType);
      $.CONSUME(RParen);
    });
  }

  private variantParser() {
    const $ = this as SailsParser & Record<string, any>;

    $.RULE('variant', () => {
      $.CONSUME(Variant);
      $.CONSUME(LCurly);
      $.AT_LEAST_ONE({
        DEF: () => $.SUBRULE($.variantField),
      });
      $.CONSUME(RCurly);
    });

    $.RULE('variantField', () => {
      $.SUBRULE($.fieldName);
      $.OPTION(() => {
        $.CONSUME(Colon);
        $.SUBRULE($.fieldType);
      });
      $.OPTION1(() => $.CONSUME(Comma));
    });
  }

  private servicesParser() {
    const $ = this as SailsParser & Record<string, any>;

    $.RULE('service', () => {
      $.CONSUME(Service);
      $.CONSUME(LCurly);
      $.MANY({
        DEF: () => $.OR([{ ALT: () => $.SUBRULE($.message) }, { ALT: () => $.SUBRULE($.query) }]),
      });
      $.CONSUME(RCurly);
      $.OPTION(() => $.CONSUME(Semicolon));
    });

    $.RULE('message', () => {
      $.CONSUME(Message);
      $.SUBRULE($.methodName);
      $.CONSUME(Colon);
      $.SUBRULE($.methodArguments);
      $.CONSUME(Arrow);
      $.SUBRULE($.methodOutput);
      $.CONSUME(Semicolon);
    });

    $.RULE('query', () => {
      $.CONSUME(Query);
      $.SUBRULE($.methodName);
      $.CONSUME(Colon);
      $.SUBRULE($.methodArguments);
      $.CONSUME(Arrow);
      $.SUBRULE($.methodOutput);
      $.CONSUME(Semicolon);
    });
  }

  private methodParser() {
    const $ = this as SailsParser & Record<string, any>;

    $.RULE('methodName', () => {
      $.CONSUME(Identifier);
    });

    $.RULE('methodArguments', () => {
      $.CONSUME(LParen);
      $.OPTION(() => {
        $.SUBRULE($.argument);
        $.MANY(() => {
          $.CONSUME(Comma);
          $.SUBRULE1($.argument);
        });
      });
      $.CONSUME(RParen);
    });

    $.RULE('argument', () => {
      $.SUBRULE($.argumentName);
      $.CONSUME(Colon);
      $.SUBRULE($.argumentType);
    });

    $.RULE('argumentName', () => $.CONSUME(Identifier));

    $.RULE('argumentType', () => $.SUBRULE($.def));

    $.RULE('methodOutput', () => $.SUBRULE($.def));
  }
}

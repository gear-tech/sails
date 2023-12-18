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
  Enum,
} from './tokens.js';

export class SailsParser extends CstParser {
  constructor() {
    super(ALL_TOKENS, { recoveryEnabled: true });

    const $ = this as SailsParser & Record<string, any>;

    this.commonParser();
    this.structParser();
    this.tupleParser();
    this.enumParser();
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
        { ALT: () => $.SUBRULE($.enum) },
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

  private enumParser() {
    const $ = this as SailsParser & Record<string, any>;

    $.RULE('enum', () => {
      $.CONSUME(Enum);
      $.CONSUME(LCurly);
      $.AT_LEAST_ONE({
        DEF: () => $.SUBRULE($.enumField),
      });
      $.CONSUME(RCurly);
    });

    $.RULE('enumField', () => {
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
        DEF: () => $.OR([{ ALT: () => $.SUBRULE($.query) }, { ALT: () => $.SUBRULE($.message) }]),
      });
      $.CONSUME(RCurly);
      $.OPTION(() => $.CONSUME(Semicolon));
    });

    $.RULE('message', () => {
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

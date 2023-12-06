import { ITokenConfig, Lexer, TokenType, createToken as orgCreateToken } from 'chevrotain';

export const ALL_TOKENS: TokenType[] = [];

function createToken(config: ITokenConfig) {
  const token = orgCreateToken(config);
  ALL_TOKENS.push(token);
  return token;
}

createToken({
  name: 'WhiteSpace',
  pattern: /\s+/,
  group: Lexer.SKIPPED,
});

createToken({
  name: 'LineTerminator',
  pattern: /[\n\r\t]+/,
  group: Lexer.SKIPPED,
});

export const Semicolon = createToken({ name: 'Semicolon', pattern: ';' });

export const LParen = createToken({ name: 'LParen', pattern: '(' });

export const RParen = createToken({ name: 'RParen', pattern: ')' });

export const LCurly = createToken({ name: 'LCurly', pattern: '{' });

export const RCurly = createToken({ name: 'RCurly', pattern: '}' });

export const LAngle = createToken({ name: 'LAngle', pattern: '<' });

export const RAngle = createToken({ name: 'RAngle', pattern: '>' });

export const Colon = createToken({ name: 'Colon', pattern: /:/ });

export const Comma = createToken({ name: 'Comma', pattern: /,/ });

export const Equals = createToken({ name: 'Equals', pattern: /=/ });

export const Arrow = createToken({ name: 'Arrow', pattern: /->/ });

export const Type = createToken({ name: 'Type', pattern: /type/ });

export const Service = createToken({ name: 'Service', pattern: /service/ });

export const Message = createToken({ name: 'Message', pattern: /async/ });

export const Query = createToken({ name: 'Query', pattern: /query/ });

export const Struct = createToken({ name: 'Struct', pattern: /struct/ });

export const Option = createToken({ name: 'Option', pattern: /opt/ });

export const Vec = createToken({ name: 'Vec', pattern: /vec/ });

export const Result = createToken({ name: 'Result', pattern: /result/ });

export const Variant = createToken({ name: 'Variant', pattern: /variant/ });

export const Identifier = createToken({ name: 'Identifier', pattern: /[A-Za-z0-9]+/ });

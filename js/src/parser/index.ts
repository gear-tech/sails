import { SailsLexer } from './lexer.js';
import { SailsParser } from './parser.js';
import { getSailsVisitorClass } from './visitor.js';

export function parse(text: string) {
  const parser = new SailsParser();

  const SailsVisitor = getSailsVisitorClass(parser);

  const visitor = new SailsVisitor();

  const lexResult = SailsLexer.tokenize(text);

  parser.input = lexResult.tokens;

  const cst = parser.parse();

  if (parser.errors.length > 0) {
    console.log(parser.errors);
    throw new Error('Parsing errors detected!');
  }

  const result = visitor.visit(cst);

  return result;
}

use logos::{Logos, SpannedIter};
use std::fmt::{Display, Formatter, Result as FmtResult};

pub(crate) type Spanned<Token, Location, Error> = Result<(Location, Token, Location), Error>;

#[derive(Debug)]
pub(crate) enum LexicalError {
    InvalidToken,
}

pub(crate) struct Lexer<'input> {
    // instead of an iterator over characters, we have a token iterator
    token_stream: SpannedIter<'input, Token>,
}

impl<'input> Lexer<'input> {
    pub fn new(input: &'input str) -> Self {
        // the Token::lexer() method is provided by the Logos trait
        Self {
            token_stream: Token::lexer(input).spanned(),
        }
    }
}

impl<'input> Iterator for Lexer<'input> {
    type Item = Spanned<Token, usize, LexicalError>;

    fn next(&mut self) -> Option<Self::Item> {
        self.token_stream.next().map(|(token, span)| match token {
            Err(_) => Err(LexicalError::InvalidToken),
            Ok(token) => Ok((span.start, token, span.end)),
        })
    }
}

#[derive(Logos, Debug, Clone, PartialOrd, Ord, PartialEq, Eq)]
#[logos(skip r"[ \t\r\n]+")] // token separators
pub(crate) enum Token {
    #[token("=")]
    Equals,
    #[token("(")]
    LParen,
    #[token(")")]
    RParen,
    #[token("{")]
    LBrace,
    #[token("}")]
    RBrace,
    #[token("[")]
    LBracket,
    #[token("]")]
    RBracket,
    #[token(";")]
    Semicolon,
    #[token(":")]
    Colon,
    #[token(",")]
    Comma,
    #[token("->")]
    Arrow,
    #[token("null")]
    Null,
    #[token("struct")]
    Struct,
    #[token("enum")]
    Enum,
    #[token("service")]
    Service,
    #[token("query")]
    Query,
    #[token("type")]
    Type,
    #[token("opt")]
    Opt,
    #[token("result")]
    Result,
    #[token("vec")]
    Vec,
    #[token("map")]
    Map,
    #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*", |lex| lex.slice().to_string())]
    Id(String),
    #[regex(r"[0-9]+", |lex| lex.slice().parse().ok())]
    Num(u32),
}

impl Display for Token {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> FmtResult {
        write!(fmt, "{:?}", self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lexer_works() {
        let lex = Token::lexer(
            r"
          type ThisThatSvcAppTupleStruct = struct {
            bool,
          };

          type ThisThatSvcAppDoThatParam = struct {
            p1: u32,
            p2: str,
            p3: ThisThatSvcAppManyVariants,
          };

          type ThisThatSvcAppManyVariants = enum {
            One,
            Two: u32,
            Three: opt u32,
            Four: struct { a: u32, b: opt u16 },
            Five: struct { str, u32 },
            Six: struct { u32 },
            Seven: [map (u32, str), 10],
          };

          service {
            DoThis : (p1: u32, p2: str, p3: struct { opt str, u8 }, p4: ThisThatSvcAppTupleStruct) -> struct { str, u32 };
            DoThat : (param: ThisThatSvcAppDoThatParam) -> result (struct { str, u32 }, struct { str });
            query This : () -> u32;
            query That : () -> result (str, str);
          }
          ",
        );
        for result in lex {
            match result {
                //Ok(token) => println!("{:#?}", token),
                Ok(_token) => (),
                Err(e) => panic!("some error occured: {:?}", e),
            }
        }
    }
}

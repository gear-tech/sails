use grammar::ProgramParser;
use lexer::Lexer;

mod grammar;
mod lexer;
mod types;

#[no_mangle]
pub extern "C" fn parse_idl_from_utf8(idl_data: *const u8, idl_len: u32) {
    let idl = unsafe { std::slice::from_raw_parts(idl_data, idl_len.try_into().unwrap()) };
    let idl = std::str::from_utf8(idl).unwrap();
    let _program = parse_idl_from_str(idl).unwrap();
}

pub fn parse_idl_from_str(idl: &str) -> Result<types::Program, String> {
    let lexer = Lexer::new(idl);
    let parser = ProgramParser::new();
    let program = parser.parse(lexer).map_err(|e| format!("{:?}", e))?;
    Ok(program)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parser_works() {
        let lexer = lexer::Lexer::new(
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
        };

        service {
          DoThis : (p1: u32, p2: str, p3: struct { opt str, u8 }, p4: ThisThatSvcAppTupleStruct) -> struct { str, u32 },
          DoThat : (param: ThisThatSvcAppDoThatParam) -> result (struct { str, u32 }, struct { str }),
          query This : (v1: vec u16) -> u32,
          query That : (v1: null) -> result (str, str),
        }
        ",
        );

        let parser = grammar::ProgramParser::new();

        let program = parser.parse(lexer).unwrap();

        assert_eq!(program.items.len(), 4);

        //println!("ast: {:#?}", program);
    }
}

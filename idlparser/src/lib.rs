use grammar::ProgramParser;
use lexer::Lexer;
use std::{slice, str};

mod grammar;
mod lexer;
mod types;

/// # Safety
///
/// See the safity documentation of [`std::slice::from_raw_parts`].
#[no_mangle]
pub unsafe extern "C" fn parse_idl_from_utf8(idl_data: *const u8, idl_len: u32) {
    let idl = unsafe { slice::from_raw_parts(idl_data, idl_len.try_into().unwrap()) };
    let idl = str::from_utf8(idl).unwrap();
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
        let program_idl = r"
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
          };
  
          type T = enum { One }
        ";

        let program = parse_idl_from_str(program_idl).unwrap();

        assert_eq!(program.types.len(), 4);

        //println!("ast: {:#?}", program);
    }

    #[test]
    fn parser_requires_service() {
        let program_idl = r"
          type T = enum { One };
        ";

        let program = parse_idl_from_str(program_idl);

        assert!(program.is_err());
    }

    #[test]
    fn parser_requires_single_service() {
        let program_idl = r"
          service {};
          service {}
        ";

        let program = parse_idl_from_str(program_idl);

        assert!(program.is_err());
    }

    #[test]
    fn parser_accepts_types_service() {
        let program_idl = r"
          type T = enum { One };
          service {}
        ";

        let program = parse_idl_from_str(program_idl).unwrap();

        assert_eq!(program.types.len(), 1);
    }

    #[test]
    fn parser_requires_semicolon_between_types_and_service() {
        let program_idl = r"
          type T = enum { One }
          service {}
        ";

        let program = parse_idl_from_str(program_idl);

        assert!(program.is_err());
    }

    #[test]
    fn parser_accepts_service_types() {
        let program_idl = r"
          service {};
          type T = enum { One };
        ";

        let program = parse_idl_from_str(program_idl).unwrap();

        assert_eq!(program.types.len(), 1);
    }

    #[test]
    fn parser_requires_semicolon_between_service_and_types() {
        let program_idl = r"
          service {}
          type T = enum { One };
        ";

        let program = parse_idl_from_str(program_idl);

        assert!(program.is_err());
    }
}

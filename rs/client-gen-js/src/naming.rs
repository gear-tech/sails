use convert_case::{Case, Casing};

const RESERVED: &[&str] = &[
    "abstract",
    "any",
    "break",
    "case",
    "catch",
    "class",
    "const",
    "continue",
    "debugger",
    "declare",
    "default",
    "delete",
    "do",
    "else",
    "enum",
    "export",
    "extends",
    "false",
    "finally",
    "for",
    "from",
    "function",
    "get",
    "if",
    "infer",
    "is",
    "keyof",
    "import",
    "in",
    "instanceof",
    "interface",
    "let",
    "module",
    "namespace",
    "never",
    "new",
    "null",
    "number",
    "object",
    "of",
    "return",
    "readonly",
    "require",
    "global",
    "set",
    "super",
    "switch",
    "symbol",
    "this",
    "throw",
    "true",
    "try",
    "type",
    "typeof",
    "undefined",
    "unique",
    "unknown",
    "void",
    "while",
    "with",
    "yield",
    "as",
    "implements",
    "package",
    "private",
    "protected",
    "public",
    "static",
];

pub(crate) fn to_camel(input: &str) -> String {
    input.to_case(Case::Camel)
}

pub(crate) fn escape_ident(input: &str) -> String {
    if RESERVED.contains(&input) {
        format!("${input}")
    } else {
        input.to_string()
    }
}

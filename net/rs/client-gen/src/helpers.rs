use convert_case::Casing;
use genco::{
    lang::{csharp::Tokens, Csharp},
    tokens::{FormatInto, ItemStr},
};
use sails_idl_parser::ast::FuncParam;

pub(crate) fn encoded_fn_args_comma_prefixed(params: &[FuncParam]) -> String {
    params
        .iter()
        .map(|p| {
            format!(
                ", {}",
                escape_keywords(p.name().to_case(convert_case::Case::Camel))
            )
        })
        .collect::<Vec<_>>()
        .join("")
}

pub fn summary_comment<T>(comment: T) -> SummaryComment<T>
where
    T: IntoIterator,
    T::Item: Into<ItemStr>,
{
    SummaryComment(comment)
}

pub struct SummaryComment<T>(pub T);

impl<T> FormatInto<Csharp> for SummaryComment<T>
where
    T: IntoIterator,
    T::Item: Into<ItemStr>,
{
    fn format_into(self, tokens: &mut Tokens) {
        let mut iter = self.0.into_iter().peekable();
        if iter.peek().is_none() {
            return;
        }
        tokens.push();
        tokens.append(ItemStr::Static("/// <summary>"));
        for line in iter {
            tokens.push();
            tokens.append(ItemStr::Static("///"));
            tokens.space();
            tokens.append(line.into());
        }
        tokens.push();
        tokens.append(ItemStr::Static("/// </summary>"));
        tokens.push();
    }
}

pub fn inheritdoc() -> InheritDoc {
    InheritDoc
}

pub struct InheritDoc;

impl FormatInto<Csharp> for InheritDoc {
    fn format_into(self, tokens: &mut Tokens) {
        tokens.push();
        tokens.append(ItemStr::Static("/// <inheritdoc/>"));
        tokens.push();
    }
}

pub fn escape_keywords(name: String) -> String {
    if RESERVED_KEYWORDS.contains(&name.as_str()) {
        format!("@{name}")
    } else {
        name
    }
}

const RESERVED_KEYWORDS: &[&str] = &[
    "abstract",
    "as",
    "base",
    "bool",
    "break",
    "byte",
    "case",
    "catch",
    "char",
    "checked",
    "class",
    "const",
    "continue",
    "decimal",
    "default",
    "delegate",
    "do",
    "double",
    "else",
    "enum",
    "event",
    "explicit",
    "extern",
    "false",
    "finally",
    "fixed",
    "float",
    "for",
    "foreach",
    "goto",
    "if",
    "implicit",
    "in",
    "int",
    "interface",
    "internal",
    "is",
    "lock",
    "long",
    "namespace",
    "new",
    "null",
    "object",
    "operator",
    "out",
    "override",
    "params",
    "private",
    "protected",
    "public",
    "readonly",
    "ref",
    "return",
    "sbyte",
    "sealed",
    "short",
    "sizeof",
    "stackalloc",
    "static",
    "string",
    "struct",
    "switch",
    "this",
    "throw",
    "true",
    "try",
    "typeof",
    "uint",
    "ulong",
    "unchecked",
    "unsafe",
    "ushort",
    "using",
    "virtual",
    "void",
    "volatile",
    "while",
];

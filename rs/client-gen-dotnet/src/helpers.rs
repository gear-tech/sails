use convert_case::Casing;
use genco::{
    lang::{csharp::Tokens, Csharp},
    tokens::{FormatInto, ItemStr},
};
use parity_scale_codec::Encode;
use sails_idl_parser::ast::FuncParam;

pub(crate) fn path_bytes(path: &str) -> (String, usize) {
    if path.is_empty() {
        (String::new(), 0)
    } else {
        let service_path_bytes = path.encode();
        let service_path_encoded_length = service_path_bytes.len();
        let service_path_bytes = service_path_bytes
            .into_iter()
            .map(|x| x.to_string())
            .collect::<Vec<_>>()
            .join(", ");

        (service_path_bytes, service_path_encoded_length)
    }
}

pub(crate) fn encoded_fn_args(params: &[FuncParam]) -> String {
    params
        .iter()
        .map(|a| a.name().to_case(convert_case::Case::Camel))
        .collect::<Vec<_>>()
        .join(", ")
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

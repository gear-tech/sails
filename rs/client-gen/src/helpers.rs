use crate::type_generators::generate_type_decl_code;
use parity_scale_codec::Encode;
use sails_idl_parser::ast::FuncParam;

pub(crate) fn path_bytes(path: &str) -> (String, usize) {
    if path.is_empty() {
        (String::new(), 0)
    } else {
        let service_path_bytes = path.encode();
        let service_path_encoded_length = service_path_bytes.len();
        let mut service_path_bytes = service_path_bytes
            .into_iter()
            .map(|x| x.to_string())
            .collect::<Vec<_>>()
            .join(",");

        service_path_bytes.push(',');

        (service_path_bytes, service_path_encoded_length)
    }
}

pub(crate) fn method_bytes(fn_name: &str) -> (String, usize) {
    let route_bytes = fn_name.encode();
    let route_encoded_length = route_bytes.len();
    let route_bytes = route_bytes
        .into_iter()
        .map(|x| x.to_string())
        .collect::<Vec<_>>()
        .join(",");

    (route_bytes, route_encoded_length)
}

pub(crate) fn fn_args(params: &[FuncParam]) -> String {
    params
        .iter()
        .map(|a| a.name())
        .collect::<Vec<_>>()
        .join(", ")
}

pub(crate) fn encoded_args(params: &[FuncParam]) -> String {
    let sep = if params.len() == 1 { "," } else { "" };
    let arg_names = fn_args(params);

    format!("({arg_names}{sep})")
}

pub(crate) fn fn_args_with_types(params: &[FuncParam]) -> String {
    params
        .iter()
        .map(|p| {
            let ty = generate_type_decl_code(p.type_decl());
            format!("{}: {}", p.name(), ty)
        })
        .collect::<Vec<_>>()
        .join(", ")
}

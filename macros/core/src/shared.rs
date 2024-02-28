use proc_macro_error::abort;
use quote::ToTokens;
use syn::{
    spanned::Spanned, FnArg, Ident, ItemImpl, Pat, Receiver, ReturnType, Signature, Type, TypePath,
};

/// A struct that represents the type of an `impl` block.
pub(crate) struct ImplType<'a> {
    path: &'a TypePath,
}

impl<'a> ImplType<'a> {
    pub(crate) fn new(r#impl: &'a ItemImpl) -> Self {
        let impl_type = r#impl.self_ty.as_ref();
        let path = if let Type::Path(type_path) = impl_type {
            type_path
        } else {
            abort!(
                impl_type.span(),
                "Failed to parse item type: {}",
                impl_type.to_token_stream()
            )
        };
        Self { path }
    }

    pub(crate) fn path(&self) -> &TypePath {
        self.path
    }
}

/// Represents parts of a handler function.
pub(crate) struct Handler<'a> {
    func: &'a Ident,
    receiver: Option<&'a Receiver>,
    params: Vec<(&'a Ident, &'a Type)>,
    result: &'a Type,
    is_async: bool,
}

impl<'a> Handler<'a> {
    pub(crate) fn from(handler_signature: &'a Signature) -> Self {
        let func = &handler_signature.ident;
        let receiver = handler_signature.receiver();
        let params = Self::extract_params(handler_signature).collect();
        let result = Self::extract_result(handler_signature);
        Self {
            func,
            receiver,
            params,
            result,
            is_async: handler_signature.asyncness.is_some(),
        }
    }

    pub(crate) fn func(&self) -> &Ident {
        self.func
    }

    pub(crate) fn receiver(&self) -> Option<&Receiver> {
        self.receiver
    }

    pub(crate) fn params(&self) -> &[(&Ident, &Type)] {
        &self.params
    }

    pub(crate) fn result(&self) -> &Type {
        self.result
    }

    pub(crate) fn is_async(&self) -> bool {
        self.is_async
    }

    fn extract_params(handler_signature: &Signature) -> impl Iterator<Item = (&Ident, &Type)> {
        handler_signature.inputs.iter().filter_map(|arg| {
            if let FnArg::Typed(arg) = arg {
                let arg_ident = if let Pat::Ident(arg_ident) = arg.pat.as_ref() {
                    &arg_ident.ident
                } else {
                    abort!(arg.span(), "Unnamed arguments are not supported");
                };
                return Some((arg_ident, arg.ty.as_ref()));
            }
            None
        })
    }

    fn extract_result(handler_signature: &Signature) -> &Type {
        if let ReturnType::Type(_, ty) = &handler_signature.output {
            ty.as_ref()
        } else {
            abort!(
                handler_signature.output.span(),
                "Failed to parse return type"
            );
        }
    }
}

use proc_macro_error::abort;
use proc_macro2::TokenStream;
use syn::{
    ImplItemFn, LitInt, Path, Token,
    parse::{Parse, ParseStream},
    spanned::Spanned,
};

#[derive(Clone, Debug)]
pub(crate) struct OverrideInfo {
    pub target: OverrideTarget,
}

#[derive(Clone, Debug)]
pub(crate) enum OverrideTarget {
    Type(Path),
    Manual { interface: Path, entry_id: u16 },
}

impl Parse for OverrideInfo {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let first_arg = input.parse::<Path>()?;
        let mut target = OverrideTarget::Type(first_arg.clone());

        if input.peek(Token![,]) && input.peek2(LitInt) {
            input.parse::<Token![,]>()?;
            let lit = input.parse::<LitInt>()?;
            let entry_id = lit.base10_parse::<u16>()?;
            target = OverrideTarget::Manual {
                interface: first_arg,
                entry_id,
            };
        }
        Ok(Self { target })
    }
}

pub(crate) fn invocation_override(fn_impl: &ImplItemFn) -> Option<OverrideInfo> {
    fn_impl
        .attrs
        .iter()
        .find(|attr| attr.path().is_ident("override_entry"))
        .map(|attr| {
            let list = attr.meta.require_list().unwrap_or_else(|_| {
                abort!(
                    attr.span(),
                    "failed to parse `override_entry` attribute: expected `#[override_entry(...)]`"
                )
            });
            list.parse_args_with(OverrideInfo::parse)
                .unwrap_or_else(|err| {
                    abort!(
                        list.span(),
                        "failed to parse `override_entry` attribute: {}",
                        err
                    )
                })
        })
}

pub fn override_entry(args: TokenStream, impl_item_fn_tokens: TokenStream) -> TokenStream {
    let fn_impl: ImplItemFn = syn::parse2::<ImplItemFn>(impl_item_fn_tokens.clone())
        .unwrap_or_else(|err| {
            abort!(
                err.span(),
                "`override_entry` attribute can be applied to methods only: {}",
                err
            )
        });

    if !args.is_empty() {
        let _ = syn::parse2::<OverrideInfo>(args).unwrap_or_else(|err| {
            abort!(
                fn_impl.span(),
                "`override_entry` attribute cannot be parsed: {}",
                err
            )
        });
    }

    impl_item_fn_tokens
}

use crate::{helpers::*, type_decl_generators::*};
use convert_case::{Case, Casing};
use csharp::Tokens;
use genco::prelude::*;
use sails_idl_parser::{ast::visitor, ast::visitor::Visitor, ast::*};

pub(crate) struct CtorFactoryGenerator<'a> {
    service_name: String,
    type_generator: TypeDeclGenerator<'a>,
    class_tokens: Tokens,
    interface_tokens: Tokens,
}

impl<'a> CtorFactoryGenerator<'a> {
    pub(crate) fn new(service_name: String, type_generator: TypeDeclGenerator<'a>) -> Self {
        Self {
            service_name,
            type_generator,
            class_tokens: Tokens::new(),
            interface_tokens: Tokens::new(),
        }
    }

    pub(crate) fn finalize(self) -> Tokens {
        let class_name = format!("{}Factory", self.service_name);
        let remoting = &csharp::import("global::Sails.Remoting.Abstractions.Core", "IRemoting");

        quote! {
            public interface I$(&class_name)$['\r']
            {
                $(self.interface_tokens)
            }
            $['\n']
            public sealed partial class $(&class_name) : I$(&class_name)$['\r']
            {$['\n']
                private readonly $remoting remoting;
                $['\n']
                public $(&class_name)($remoting remoting)$['\r']
                {
                    this.remoting = remoting;
                }
                $['\n']
                $(self.class_tokens)
            }
            $['\n']
        }
    }
}

impl<'a> Visitor<'a> for CtorFactoryGenerator<'a> {
    fn visit_ctor(&mut self, ctor: &'a Ctor) {
        visitor::accept_ctor(ctor, self);
    }

    fn visit_ctor_func(&mut self, func: &'a CtorFunc) {
        let func_name_pascal = &func.name().to_case(Case::Pascal);

        self.interface_tokens.push();
        self.interface_tokens.append(summary_comment(func.docs()));
        self.interface_tokens.push();

        let route_bytes = &path_bytes(func.name()).0;
        let args = &encoded_fn_args_comma_prefixed(func.params());
        let args_with_type = &self.type_generator.fn_params_with_types(func.params());
        let void_type = primitive_type_to_dotnet(PrimitiveType::Null);

        let activation = &csharp::import("global::Sails.Remoting.Abstractions", "IActivation");
        let action = &csharp::import("global::Sails.Remoting", "RemotingAction");

        quote_in! { self.interface_tokens =>
            $activation $func_name_pascal($args_with_type);$['\r']
        };

        quote_in! { self.class_tokens =>
            $(inheritdoc())
            public $activation $func_name_pascal($args_with_type)
            {
                return new $action<$(void_type)>(this.remoting, [$route_bytes]$args);
            }
            $['\n']
        };
    }
}

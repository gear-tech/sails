use crate::{helpers::*, type_decl_generators::*};
use convert_case::{Case, Casing};
use csharp::Tokens;
use genco::prelude::*;
use sails_idl_parser::{ast::visitor, ast::visitor::Visitor, ast::*};

pub(crate) struct EventsGenerator<'a> {
    service_name: &'a str,
    type_generator: TypeDeclGenerator<'a>,
    enum_tokens: Tokens,
    class_tokens: Tokens,
    event_routes_tokens: Tokens,
}

impl<'a> EventsGenerator<'a> {
    pub(crate) fn new(service_name: &'a str, type_generator: TypeDeclGenerator<'a>) -> Self {
        Self {
            service_name,
            type_generator,
            enum_tokens: Tokens::new(),
            class_tokens: Tokens::new(),
            event_routes_tokens: Tokens::new(),
        }
    }

    pub(crate) fn finalize(self) -> Tokens {
        let name = self.service_name;
        let enum_name = &format!("{}Events", name);
        let class_name = &format!("Enum{}Events", name);
        let listener_name = &format!("{}Listener", name);

        let remoting = &csharp::import("global::Sails.Remoting.Abstractions.Core", "IRemoting");
        let task = &csharp::import("global::System.Threading.Tasks", "Task");
        let cancellation_token = &csharp::import("global::System.Threading", "CancellationToken");
        let listener = &csharp::import("global::Sails.Remoting.Abstractions.Core", "EventListener");
        let actor_id_type = primitive_type_to_dotnet(PrimitiveType::ActorId);

        quote! {
            public enum $enum_name
            {
                $(self.enum_tokens)
            }
            $['\n']
            public sealed partial class $class_name : global::Substrate.NetApi.Model.Types.Base.BaseEnumRust<$enum_name>
            {
                public $class_name()
                {
                    $(self.class_tokens)
                }
            }
            $['\n']
            public sealed partial class $listener_name
            {
                $['\n']
                private const string ROUTE = $(quoted(name));
                $['\n']
                private static readonly string[] EventRoutes =
                [
                    $(self.event_routes_tokens)
                ];
                $['\n']
                private readonly $remoting remoting;
                $['\n']
                public $listener_name($remoting remoting)
                {
                    this.remoting = remoting;
                }
                $['\n']
                public async $task<$listener<($actor_id_type, $class_name)>> ListenAsync($cancellation_token cancellationToken = default)
                {$['\r']
                    var listener = await this.remoting.ListenAsync(cancellationToken);$['\r']
                    return listener.SelectEvents<$class_name>(ROUTE, EventRoutes);$['\r']
                }
            }
            $['\n']
        }
    }
}

impl<'a> Visitor<'a> for EventsGenerator<'a> {
    fn visit_service(&mut self, service: &'a Service) {
        visitor::accept_service(service, self);
    }

    fn visit_service_event(&mut self, event: &'a ServiceEvent) {
        let name = &self.service_name.to_case(Case::Pascal);

        quote_in! { self.event_routes_tokens =>
            $(quoted(event.name())),
        };

        quote_in! { self.enum_tokens =>
            $(summary_comment(event.docs()))
            $(event.name()),$['\r']
        };

        let type_decl_code = if let Some(type_decl) = event.type_decl().as_ref() {
            self.type_generator.generate_type_decl(type_decl)
        } else {
            primitive_type_to_dotnet(PrimitiveType::Null).into()
        };
        quote_in! { self.class_tokens =>
            this.AddTypeDecoder<$(type_decl_code)>($(name)Events.$(event.name()));$['\r']
        }
    }
}

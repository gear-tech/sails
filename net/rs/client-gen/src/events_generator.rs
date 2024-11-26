use crate::helpers::*;
use crate::type_generators::{primitive_type_to_dotnet, TypeDeclGenerator};
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

        let core_listener = &csharp::import(
            "global::Sails.Remoting.Abstractions.Core",
            "IRemotingListener",
        );
        let service_listener =
            &csharp::import("global::Sails.Remoting.Abstractions", "IRemotingListener");
        let client_listener = &csharp::import("global::Sails.Remoting", "RemotingListener");
        let task = &csharp::import("global::System.Threading.Tasks", "Task");
        let cancellation_token = &csharp::import("global::System.Threading", "CancellationToken");

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
            public static class $listener_name
            {
                $['\n']
                private const string ROUTE = $(quoted(name));
                $['\n']
                private static readonly string[] EventRoutes =
                [
                    $(self.event_routes_tokens)
                ];
                $['\n']
                public static async $task<$service_listener<$class_name>> SubscribeAsync($core_listener remoting, $cancellation_token cancellationToken = default)
                {$['\r']
                    var eventStream = await remoting.ListenAsync(cancellationToken);$['\r']
                    return new $client_listener<$class_name>(eventStream, ROUTE, EventRoutes);$['\r']
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

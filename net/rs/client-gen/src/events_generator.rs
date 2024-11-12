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
    listener_tokens: Tokens,
}

impl<'a> EventsGenerator<'a> {
    pub(crate) fn new(service_name: &'a str, type_generator: TypeDeclGenerator<'a>) -> Self {
        Self {
            service_name,
            type_generator,
            enum_tokens: Tokens::new(),
            class_tokens: Tokens::new(),
            listener_tokens: Tokens::new(),
        }
    }

    pub(crate) fn finalize(self) -> Tokens {
        let name = &self.service_name.to_case(Case::Pascal);
        let enum_name = &format!("{}Events", name);
        let class_name = &format!("Enum{}Events", name);
        let listener_name = &format!("{}Listener", name);

        let system_buffer = &csharp::import("global::System", "Buffer");
        let core_listener = &csharp::import(
            "global::Sails.Remoting.Abstractions.Core",
            "IRemotingListener",
        );
        let service_listener =
            &csharp::import("global::Sails.Remoting.Abstractions", "IRemotingListener");

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
            public sealed partial class $listener_name : $service_listener<$class_name>
            {
                private static readonly byte[][] EventRoutes =
                [
                    $(self.listener_tokens)
                ];
                $['\n']
                private readonly $core_listener remoting;
                $['\n']
                public $listener_name($core_listener remoting)
                {
                    this.remoting = remoting;
                }
                $['\n']
                public async global::System.Collections.Generic.IAsyncEnumerable<$class_name> ListenAsync([global::System.Runtime.CompilerServices.EnumeratorCancellation] global::System.Threading.CancellationToken cancellationToken = default)
                {
                    await foreach (var bytes in this.remoting.ListenAsync(cancellationToken))
                    {
                        byte idx = 0;
                        foreach (var route in EventRoutes)
                        {
                            if (route.Length > bytes.Length)
                            {
                                continue;
                            }
                            if (route.AsSpan().SequenceEqual(bytes.AsSpan()[..route.Length]))
                            {
                                var bytesLength = bytes.Length - route.Length + 1;
                                var data = new byte[bytesLength];
                                data[0] = idx;
                                $system_buffer.BlockCopy(bytes, route.Length, data, 1, bytes.Length - route.Length);

                                var p = 0;
                                $class_name ev = new();
                                ev.Decode(bytes, ref p);
                                yield return ev;
                            }
                            idx++;
                        }
                    }
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
        let service_route_bytes = path_bytes(self.service_name).0;
        let event_route_bytes = path_bytes(event.name()).0;
        let route_bytes = [service_route_bytes, event_route_bytes].join(", ");

        quote_in! { self.listener_tokens =>
            [$(&route_bytes)],
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

// This file is part of Gear.

// Copyright (C) 2021-2023 Gear Technologies Inc.
// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

//! Implemntation of the procedural macros exposed via the `gprogram-framework-macros` crate.

use proc_macro2::TokenStream as TokenStream2;

mod processors;

const COMMAND_ENUM_NAME: &str = "Commands";
const COMMAND_RESPONSES_ENUM_NAME: &str = "CommandResponses";
const QUERY_ENUM_NAME: &str = "Queries";
const QUERY_RESPONSES_ENUM_NAME: &str = "QueryResponses";

pub fn command_handlers_core(mod_tokens: TokenStream2) -> TokenStream2 {
    processors::generate(mod_tokens, COMMAND_ENUM_NAME, COMMAND_RESPONSES_ENUM_NAME)
}

pub fn query_handlers_core(mod_tokens: TokenStream2) -> TokenStream2 {
    processors::generate(mod_tokens, QUERY_ENUM_NAME, QUERY_RESPONSES_ENUM_NAME)
}

pub fn gservice_core(impl_tokens: TokenStream2) -> TokenStream2 {
    processors::gservice(impl_tokens)
}

#[cfg(test)]
mod tests {
    use super::*;
    use quote::quote;

    #[test]
    fn gservice_core_works() {
        let input = quote! {
            impl SomeService {
                pub async fn do_this(&mut self, p1: u32, p2: String) -> u32 {
                    p1
                }

                pub fn this(&self, p1: bool) -> bool {
                    p1
                }
            }
        };
        let expected = quote!(
            impl SomeService {
                pub async fn do_this(&mut self, p1: u32, p2: String) -> u32 {
                    p1
                }
                pub fn this(&self, p1: bool) -> bool {
                    p1
                }
            }

            #[derive(Decode, TypeInfo)]
            pub struct DoThisParams {
                p1: u32,
                p2: String
            }

            #[derive(Decode, TypeInfo)]
            pub struct ThisParams {
                p1: bool
            }

            pub mod meta {
                use super::*;

                #[derive(TypeInfo)]
                pub enum CommandsMeta {
                    DoThis(DoThisParams, u32),
                }

                #[derive(TypeInfo)]
                pub enum QueriesMeta {
                    This(ThisParams, bool),
                }

                pub struct ServiceMeta;

                impl sails_service::ServiceMeta for ServiceMeta {
                    type Commands = CommandsMeta;
                    type Queries = QueriesMeta;
                }
            }

            pub mod handlers {
                use super::*;
                pub async fn process_request(service: &mut SomeService, mut input: &[u8]) -> Vec<u8> {
                    if input.starts_with("DoThis/".as_bytes()) {
                        return do_this(service, &input["DoThis/".as_bytes().len()..]).await;
                    }
                    if input.starts_with("This/".as_bytes()) {
                        return this(service, &input["This/".as_bytes().len()..]).await;
                    }
                    panic!("Unknown request");
                }
                async fn do_this(service: &mut SomeService, mut input: &[u8]) -> Vec<u8> {
                    let request = DoThisParams::decode(&mut input).expect("Failed to decode request");
                    let result = service.do_this(request.p1, request.p2).await;
                    return result.encode();
                }
                async fn this(service: &SomeService, mut input: &[u8]) -> Vec<u8> {
                    let request = ThisParams::decode(&mut input).expect("Failed to decode request");
                    let result = service.this(request.p1);
                    return result.encode();
                }
            }

        );
        assert_eq!(expected.to_string(), gservice_core(input).to_string());
    }

    #[test]
    fn command_handlers_core_works() {
        let input = quote! {
            mod commands {
                use super::*;

                struct SomeStruct {}

                fn do_this(p: SomeStruct) {}
            }
        };
        let expected = quote! {
            mod commands {
                extern crate parity_scale_codec as commands_scale_codec;
                extern crate scale_info as commands_scale_info;

                #[derive(commands_scale_codec::Encode, commands_scale_codec::Decode, commands_scale_info::TypeInfo)]
                pub enum Commands {
                    DoThis(SomeStruct,),
                }

                #[derive(commands_scale_codec::Encode, commands_scale_codec::Decode, commands_scale_info::TypeInfo)]
                pub enum CommandResponses {
                    DoThis(()),
                }

                use super::*;

                struct SomeStruct {}

                #[cfg(feature = "handlers")]
                pub mod handlers {
                    use super::*;

                    pub fn process_commands(request: Commands) -> (CommandResponses, bool) {
                        match request {
                            Commands::DoThis(v0) => {
                                let result: Result<_, _> = do_this(v0);
                                let is_error = result.is_err();
                                (CommandResponses::DoThis(result), is_error)
                            }
                        }
                    }

                    fn do_this(p: SomeStruct) {}
                }
            }
        };
        assert_eq!(
            expected.to_string(),
            command_handlers_core(input).to_string()
        );
    }

    #[test]
    fn query_handlers_core_works() {
        let input = quote! {
            pub(crate) mod queries {
                fn this() {}
            }
        };
        let expected = quote! {
            pub(crate) mod queries {
                extern crate parity_scale_codec as queries_scale_codec;
                extern crate scale_info as queries_scale_info;

                #[derive(queries_scale_codec::Encode, queries_scale_codec::Decode, queries_scale_info::TypeInfo)]
                pub enum Queries {
                    This(),
                }

                #[derive(queries_scale_codec::Encode, queries_scale_codec::Decode, queries_scale_info::TypeInfo)]
                pub enum QueryResponses {
                    This(()),
                }

                #[cfg(feature = "handlers")]
                pub mod handlers {
                    use super::*;

                    pub fn process_queries(request: Queries) -> (QueryResponses, bool) {
                        match request {
                            Queries::This() => {
                                let result: Result<_, _> = this();
                                let is_error = result.is_err();
                                (QueryResponses::This(result), is_error)
                            }
                        }
                    }

                    fn this() {}
                }
            }
        };
        assert_eq!(expected.to_string(), query_handlers_core(input).to_string());
    }
}

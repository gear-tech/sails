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

//! Implemntation of the procedural macros exposed via the `sails-macros` crate.

use proc_macro2::TokenStream as TokenStream2;

mod processors;

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

                impl sails_service_meta::ServiceMeta for ServiceMeta {
                    type Commands = CommandsMeta;
                    type Queries = QueriesMeta;
                }
            }

            pub mod requests {
                use super::*;

                pub async fn process(service: &mut SomeService, mut input: &[u8]) -> Vec<u8> {
                    let invocation_path = "DoThis".encode();
                    if input.starts_with(&invocation_path) {
                        let output = do_this(service, &input[invocation_path.len()..]).await;
                        return [invocation_path, output].concat();
                    }
                    let invocation_path = "This".encode();
                    if input.starts_with(&invocation_path) {
                        let output = this(service, &input[invocation_path.len()..]).await;
                        return [invocation_path, output].concat();
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
}

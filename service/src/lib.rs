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

//! Traits and structs for a simple service comprised of command and query dispatchers.

#![no_std]

extern crate alloc;

use alloc::{borrow::ToOwned, boxed::Box, format, string::String, vec::Vec};
use async_trait::async_trait;
use hashbrown::HashMap;
pub use meta::{BoxedFuture, CommandProcessorMeta, QueryProcessorMeta};
use parity_scale_codec::{Decode, Encode};

mod meta;

#[async_trait]
pub trait Service {
    // TODO: Make something up for error handling (some sort of Result)
    async fn process_command(&self, input: &[u8]) -> (Vec<u8>, bool);

    // TODO: Make something up for error handling (some sort of Result)
    fn process_query(&self, input: &[u8]) -> Vec<u8>;
}

pub struct SimpleService<C: CommandProcessorMeta, Q: QueryProcessorMeta> {
    process_command: C::ProcessFn,
    process_query: Q::ProcessFn,
}

impl<C: CommandProcessorMeta, Q: QueryProcessorMeta> SimpleService<C, Q> {
    pub const fn new(process_command: C::ProcessFn, process_query: Q::ProcessFn) -> Self {
        Self {
            process_command,
            process_query,
        }
    }
}

#[async_trait]
impl<C: CommandProcessorMeta, Q: QueryProcessorMeta> Service for SimpleService<C, Q> {
    async fn process_command(&self, mut input: &[u8]) -> (Vec<u8>, bool) {
        let request = C::Request::decode(&mut input).expect("Failed to decode request");
        //let (response, is_error) = (self.process_command)(request);
        // For async processing
        let (response, is_error) = (self.process_command)(request).await;
        (response.encode(), is_error)
    }

    fn process_query(&self, mut input: &[u8]) -> Vec<u8> {
        let request = Q::Request::decode(&mut input).expect("Failed to decode request");
        let (response, _) = (self.process_query)(request);
        response.encode()
    }
}

pub struct CompositeService {
    // TODO: It might be cheaper and more progmatic to use a simple Vec<(String, Box<dyn Service + Sync>)>
    //       as there is no expectation of a large number of services
    services: HashMap<String, Box<dyn Service + Sync>>,
}

impl CompositeService {
    pub fn new<'a>(services: impl IntoIterator<Item = (&'a str, Box<dyn Service + Sync>)>) -> Self {
        let services = services
            .into_iter()
            .try_fold(HashMap::new(), |mut services, service| {
                let service_route = Self::to_service_route(service.0);
                services
                    .try_insert(service_route, service.1)
                    .map_err(|e| format!("Service with name {} already exists", e.entry.key()))?;
                Ok::<_, String>(services)
            })
            .expect("Duplicate service name");
        Self { services }
    }

    fn to_service_route(name: &str) -> String {
        if name.is_empty() {
            panic!("Service name cannot be empty");
        }
        if name.contains('/') {
            panic!("Service name cannot contain '/'");
        }
        let mut service_route = name.to_owned();
        service_route.push('/');
        service_route
    }

    fn select_service_by_route<'a>(&'a self, input: &'a [u8]) -> (&(dyn Service + Sync), &[u8]) {
        self.services
            .iter()
            .find_map(|(service_route, service)| {
                if input.starts_with(service_route.as_bytes()) {
                    Some((service.as_ref(), &input[service_route.len()..]))
                } else {
                    None
                }
            })
            .expect("Service not found by route")
    }
}

#[async_trait]
impl Service for CompositeService {
    async fn process_command(&self, input: &[u8]) -> (Vec<u8>, bool) {
        let (selected_service, input) = self.select_service_by_route(input);
        selected_service.process_command(input).await
    }

    fn process_query(&self, input: &[u8]) -> Vec<u8> {
        let (selected_service, input) = self.select_service_by_route(input);
        selected_service.process_query(input)
    }
}

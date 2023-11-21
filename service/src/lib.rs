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

use alloc::{boxed::Box, vec::Vec};
use async_trait::async_trait;
use core::{future::Future, pin::Pin};
use parity_scale_codec::{Decode, Encode};
use scale_info::StaticTypeInfo;

pub type BoxedFuture<T> = Pin<Box<dyn Future<Output = T> + Send>>;

pub trait RequestProcessorMeta {
    type Request: StaticTypeInfo + Decode;
    type Response: StaticTypeInfo + Encode;
    // TODO: Make something up for error handling (some sort of Result)
    type ProcessFn: Fn(Self::Request) -> (Self::Response, bool) + Sync;
    // For async processing
    //type ProcessFn: Fn(Self::Requests) -> BoxedFuture<Self::Responses> + Sync;
}

impl RequestProcessorMeta for () {
    type Request = ();
    type Response = ();
    type ProcessFn = fn(()) -> ((), bool);
    // For async processing
    //type ProcessFn = fn(()) -> BoxedFuture<()>;
}

// TODO: Think of introducing ServiceMeta trait with associated types for command and query processors
//       Then SimpleService can have impls for the ServiceMeta and Service traits both

#[async_trait]
pub trait Service {
    // TODO: Make something up for error handling (some sort of Result)
    async fn process_command(&self, input: &[u8]) -> (Vec<u8>, bool);

    // TODO: Make something up for error handling (some sort of Result)
    fn process_query(&self, input: &[u8]) -> Vec<u8>;
}

pub struct SimpleService<C: RequestProcessorMeta, Q: RequestProcessorMeta> {
    process_command: C::ProcessFn,
    process_query: Q::ProcessFn,
}

impl<C: RequestProcessorMeta, Q: RequestProcessorMeta> SimpleService<C, Q> {
    pub const fn new(process_command: C::ProcessFn, process_query: Q::ProcessFn) -> Self {
        Self {
            process_command,
            process_query,
        }
    }
}

#[async_trait]
impl<C: RequestProcessorMeta, Q: RequestProcessorMeta> Service for SimpleService<C, Q> {
    async fn process_command(&self, mut input: &[u8]) -> (Vec<u8>, bool) {
        let request = C::Request::decode(&mut input).unwrap();
        let (response, is_error) = (self.process_command)(request);
        // For async processing
        //let response = (self.handle_command)(request).await;
        (response.encode(), is_error)
    }

    fn process_query(&self, mut input: &[u8]) -> Vec<u8> {
        let request = Q::Request::decode(&mut input).unwrap();
        let (response, _) = (self.process_query)(request);
        response.encode()
    }
}

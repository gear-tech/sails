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

//! Traits for describing request processors.

extern crate alloc;

use alloc::boxed::Box;
use core::{fmt::Debug, future::Future, pin::Pin};
use parity_scale_codec::{Decode, Encode};
use scale_info::StaticTypeInfo;

pub type BoxedFuture<T> = Pin<Box<dyn Future<Output = T> + Send>>;

pub trait CommandProcessorMeta {
    type Request: StaticTypeInfo + Decode + Debug;
    type Response: StaticTypeInfo + Encode + Debug;
    // TODO: Make something up for error handling (some sort of Result)
    type ProcessFn: Fn(Self::Request) -> BoxedFuture<(Self::Response, bool)> + Sync;
}

impl CommandProcessorMeta for () {
    type Request = ();
    type Response = ();
    type ProcessFn = fn(Self::Request) -> BoxedFuture<(Self::Response, bool)>;
}

pub trait QueryProcessorMeta {
    type Request: StaticTypeInfo + Decode;
    type Response: StaticTypeInfo + Encode;
    // TODO: Make something up for error handling (some sort of Result)
    type ProcessFn: Fn(Self::Request) -> (Self::Response, bool) + Sync;
}

impl QueryProcessorMeta for () {
    type Request = ();
    type Response = ();
    type ProcessFn = fn(Self::Request) -> (Self::Response, bool);
}

// TODO: Think of introducing ServiceMeta trait with associated types for command and query processors
//       Then SimpleService can have impls for the ServiceMeta and Service traits both

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

//! Integration tests for functionality provided by the `gprogram-framework-macros` crate.

#[test]
fn command_handlers_work() {
    let t = trybuild::TestCases::new();
    t.pass("tests/ui/command_handlers_work.rs");
}

#[test]
fn query_handlers_work() {
    let t = trybuild::TestCases::new();
    t.pass("tests/ui/query_handlers_work.rs");
}

#[test]
fn no_command_handlers() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/no_command_handlers.rs");
}

#[test]
fn no_query_handlers() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/no_query_handlers.rs");
}

#[test]
fn command_handler_returns_result() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/command_handler_returns_result.rs");
}

#[test]
fn query_handler_returns_result() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/query_handler_returns_result.rs");
}

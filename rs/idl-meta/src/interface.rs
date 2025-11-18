//! Helpers for exposing canonical interface metadata (entry ordering, kinds, etc.).
//!
//! The procedural macros will eventually embed the output of these helpers as
//! compile-time constants so runtimes can consume `(interface_id, entry_id)` pairs
//! without re-running canonicalization at runtime.

use alloc::{borrow::Cow, string::String};

#[cfg(all(feature = "ast", not(target_family = "wasm")))]
use alloc::{format, string::ToString, vec::Vec};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EntryKind {
    Command,
    Query,
    Event,
}

/// Describes an interface entry (command/query/event) together with its canonical
/// identifier.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EntryMeta<'a> {
    pub name: Cow<'a, str>,
    pub entry_id: u16,
    pub kind: EntryKind,
    pub is_async: bool,
}

impl<'a> EntryMeta<'a> {
    pub const fn borrowed(name: &'a str, entry_id: u16, kind: EntryKind, is_async: bool) -> Self {
        Self {
            name: Cow::Borrowed(name),
            entry_id,
            kind,
            is_async,
        }
    }
}

impl EntryMeta<'static> {
    pub fn owned(name: String, entry_id: u16, kind: EntryKind, is_async: bool) -> Self {
        Self {
            name: Cow::Owned(name),
            entry_id,
            kind,
            is_async,
        }
    }
}

#[cfg(all(feature = "ast", not(target_family = "wasm")))]
use crate::canonical::{
    CanonicalEnvelope, CanonicalEvent, CanonicalFunction, CanonicalFunctionKind, CanonicalType,
};

#[cfg(feature = "ast")]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CanonicalEntry {
    pub name: String,
    pub kind: EntryKind,
    pub signature: String,
}

#[cfg(all(feature = "ast", not(target_family = "wasm")))]
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum EntryAssignmentError {
    #[error("too many interface entries ({0}); entry_id is a 16-bit value")]
    TooManyEntries(usize),
}

#[cfg(all(feature = "ast", not(target_family = "wasm")))]
pub fn canonical_entries(envelope: &CanonicalEnvelope) -> Vec<CanonicalEntry> {
    let mut entries = Vec::new();
    for func in &envelope.service.functions {
        entries.push(CanonicalEntry {
            name: func.name.clone(),
            kind: match func.kind {
                CanonicalFunctionKind::Command => EntryKind::Command,
                CanonicalFunctionKind::Query => EntryKind::Query,
            },
            signature: canonical_function_signature(func),
        });
    }

    for event in &envelope.service.events {
        entries.push(CanonicalEntry {
            name: event.name.clone(),
            kind: EntryKind::Event,
            signature: canonical_event_signature(event),
        });
    }

    entries.sort_by(|lhs, rhs| {
        lhs.name
            .cmp(&rhs.name)
            .then_with(|| lhs.signature.cmp(&rhs.signature))
    });
    entries
}

#[cfg(all(feature = "ast", not(target_family = "wasm")))]
pub fn build_entry_meta_with_async(
    envelope: &CanonicalEnvelope,
    mut async_lookup: impl FnMut(&CanonicalEntry) -> bool,
) -> Result<Vec<EntryMeta<'static>>, EntryAssignmentError> {
    let entries = canonical_entries(envelope);
    if entries.len() > u16::MAX as usize + 1 {
        return Err(EntryAssignmentError::TooManyEntries(entries.len()));
    }

    Ok(entries
        .into_iter()
        .enumerate()
        .map(|(idx, entry)| {
            let is_async = async_lookup(&entry);
            EntryMeta::owned(entry.name, idx as u16, entry.kind, is_async)
        })
        .collect())
}

#[cfg(all(feature = "ast", not(target_family = "wasm")))]
fn canonical_function_signature(func: &CanonicalFunction) -> String {
    let mut signature = String::new();
    signature.push_str(match func.kind {
        CanonicalFunctionKind::Command => "command",
        CanonicalFunctionKind::Query => "query",
    });
    signature.push('|');
    signature.push_str(&join_type_list(&func.params));
    signature.push('|');
    signature.push_str(&canonical_type_repr(&func.output));
    if let Some(throws) = &func.throws {
        signature.push('|');
        signature.push_str(&canonical_type_repr(throws));
    }
    signature
}

#[cfg(all(feature = "ast", not(target_family = "wasm")))]
fn canonical_event_signature(event: &CanonicalEvent) -> String {
    join_type_list(&event.payload.fields)
}

#[cfg(all(feature = "ast", not(target_family = "wasm")))]
fn join_type_list(types: &[CanonicalType]) -> String {
    let mut acc = String::new();
    let mut first = true;
    for ty in types {
        if !first {
            acc.push(',');
        }
        first = false;
        acc.push_str(&canonical_type_repr(ty));
    }
    acc
}

#[cfg(all(feature = "ast", not(target_family = "wasm")))]
fn canonical_type_repr(ty: &CanonicalType) -> String {
    match ty {
        CanonicalType::Primitive { name } => name.to_string(),
        CanonicalType::Slice { item } => format!("[{}]", canonical_type_repr(item)),
        CanonicalType::Array { item, len } => {
            format!("[{}; {len}]", canonical_type_repr(item))
        }
        CanonicalType::Tuple { items } => {
            let mut repr = String::from("(");
            for (idx, item) in items.iter().enumerate() {
                if idx > 0 {
                    repr.push_str(", ");
                }
                repr.push_str(&canonical_type_repr(item));
            }
            repr.push(')');
            repr
        }
        CanonicalType::Option { item } => format!("Option<{}>", canonical_type_repr(item)),
        CanonicalType::Result { ok, err } => format!(
            "Result<{}, {}>",
            canonical_type_repr(ok),
            canonical_type_repr(err)
        ),
        CanonicalType::Named { type_id, args } => {
            if args.is_empty() {
                format!("type:{type_id}")
            } else {
                let mut repr = String::new();
                repr.push_str("type:");
                repr.push_str(type_id);
                repr.push('<');
                for (idx, arg) in args.iter().enumerate() {
                    if idx > 0 {
                        repr.push_str(", ");
                    }
                    repr.push_str(&canonical_type_repr(arg));
                }
                repr.push('>');
                repr
            }
        }
    }
}

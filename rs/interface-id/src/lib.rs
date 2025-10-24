use blake3::{Hash, Hasher};
use serde::Serialize;
use serde_json::to_value;
use serde_json_canonicalizer::to_vec as to_canonical_vec;

pub mod canonical;
pub mod canonical_type;
pub mod runtime;

/// Canonical schema identifier.
pub const CANONICAL_SCHEMA: &str = "sails-idl-jcs";
/// Canonical schema version.
pub const CANONICAL_VERSION: &str = "1";
/// Hash algorithm identifier used in canonical docs.
pub const CANONICAL_HASH_ALGO: &str = "blake3";
/// Domain separator (string) for interface-level hashing.
pub const INTERFACE_HASH_DOMAIN_STR: &str = "GEAR-IDL/v1/interface-id";
/// Domain separator (bytes) for interface-level hashing.
pub const INTERFACE_HASH_DOMAIN: &[u8] = b"GEAR-IDL/v1/interface-id";
/// Domain separator (string) for function/message hashing.
pub const FUNCTION_HASH_DOMAIN_STR: &str = "GEAR-IDL/v1/entry-signature";
/// Domain separator (bytes) for function/message hashing.
pub const FUNCTION_HASH_DOMAIN: &[u8] = b"GEAR-IDL/v1/entry-signature";
/// Domain separator (string) for route key hashing.
pub const ROUTE_HASH_DOMAIN_STR: &str = "GEAR-ROUTE/v1";
/// Domain separator (bytes) for route key hashing.
pub const ROUTE_HASH_DOMAIN: &[u8] = b"GEAR-ROUTE/v1";

/// Canonical description of a Sails service used to derive stable interface identifiers.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct InterfaceDescriptor {
    pub interface_path: String,
    pub extends: Vec<ExtendedInterfaceDescriptor>,
    pub commands: Vec<FunctionEntry>,
    pub queries: Vec<FunctionEntry>,
    pub events: Vec<EventEntry>,
}

/// Canonical description of a callable entry (command/query) inside a service.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct FunctionEntry {
    pub name: String,
    pub entry_id: u16,
}

/// Canonical description of an event entry emitted by a service.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct EventEntry {
    pub name: String,
    pub entry_id: u16,
}

/// Canonical description of a parent interface implemented by a service.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ExtendedInterfaceDescriptor {
    pub name: String,
    pub interface_id: u64,
}

/// Collection of interface signatures that represent a concrete service and
/// any interfaces it directly exposes via inheritance.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct InterfaceDescriptorSet {
    pub primary: InterfaceDescriptor,
    #[serde(default)]
    pub inherited: Vec<InterfaceDescriptor>,
}

impl InterfaceDescriptor {
    /// Creates an empty signature with the provided path.
    pub fn new(interface_path: impl Into<String>) -> Self {
        Self {
            interface_path: interface_path.into(),
            extends: Vec::new(),
            commands: Vec::new(),
            queries: Vec::new(),
            events: Vec::new(),
        }
    }
}

impl InterfaceDescriptorSet {
    pub fn new(primary: InterfaceDescriptor) -> Self {
        Self {
            primary,
            inherited: Vec::new(),
        }
    }

    pub fn with_inherited(mut self, inherited: Vec<InterfaceDescriptor>) -> Self {
        self.inherited = inherited;
        self
    }

    pub fn iter(&self) -> impl Iterator<Item = &InterfaceDescriptor> {
        std::iter::once(&self.primary).chain(self.inherited.iter())
    }
}

/// Computes the 64-bit interface_id using a canonical hash.
pub fn compute_ids(descriptor: &InterfaceDescriptor) -> u64 {
    let value =
        to_value(descriptor).expect("serializing interface descriptor to value should succeed");
    let canonical =
        to_canonical_vec(&value).expect("canonicalizing interface descriptor should succeed");
    compute_ids_from_bytes(&canonical)
}

/// Computes the 64-bit interface_id from a [`canonical::CanonicalDocument`].
pub fn compute_ids_from_document(doc: &canonical::CanonicalDocument) -> u64 {
    let bytes = doc
        .to_bytes()
        .expect("serializing canonical document should succeed");
    compute_ids_from_bytes(&bytes)
}

/// Computes the 64-bit interface_id from canonical bytes.
pub fn compute_ids_from_bytes(bytes: &[u8]) -> u64 {
    let digest = compute_interface_hash(bytes);
    let bytes = digest.as_bytes();
    u64::from_le_bytes(bytes[0..8].try_into().unwrap())
}

/// Computes the full interface hash (BLAKE3-256) using the canonical domain separator.
pub fn compute_interface_hash(bytes: &[u8]) -> Hash {
    blake3_hash_with_domain(INTERFACE_HASH_DOMAIN, &[bytes])
}

/// Computes the 16-bit entry identifier derived from a canonical function signature.
pub fn compute_entry_id16(
    interface_hash: &Hash,
    signature: &str,
    override_value: Option<u16>,
) -> u16 {
    if let Some(value) = override_value {
        return value;
    }

    let digest = blake3_hash_with_domain(
        FUNCTION_HASH_DOMAIN,
        &[interface_hash.as_bytes(), signature.as_bytes()],
    );
    u16::from_le_bytes(digest.as_bytes()[0..2].try_into().unwrap())
}

/// Computes a BLAKE3 hash with the provided domain separator and payload slices.
pub fn blake3_hash_with_domain(domain: &[u8], payloads: &[&[u8]]) -> Hash {
    let mut hasher = Hasher::new();
    hasher.update(domain);
    for payload in payloads {
        hasher.update(payload);
    }
    hasher.finalize()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_are_stable() {
        let mut descriptor = InterfaceDescriptor::new("example::Service");
        descriptor.extends.push(ExtendedInterfaceDescriptor {
            name: "ParentA".to_owned(),
            interface_id: 0xabcdef01_2345_6789,
        });
        descriptor.commands.push(FunctionEntry {
            name: "DoSomething".to_owned(),
            entry_id: 1,
        });
        descriptor.queries.push(FunctionEntry {
            name: "GetValue".to_owned(),
            entry_id: 2,
        });
        descriptor.events.push(EventEntry {
            name: "Occurred".to_owned(),
            entry_id: 1,
        });
        let id_a = compute_ids(&descriptor);
        let id_b = compute_ids(&descriptor);
        assert_eq!(id_a, id_b);
        assert_ne!(id_a, 0);
    }

    #[test]
    fn signature_set_iteration_includes_inherited() {
        let mut primary = InterfaceDescriptor::new("primary");
        primary.commands.push(FunctionEntry {
            name: "Foo".to_owned(),
            entry_id: 1,
        });

        let mut inherited = InterfaceDescriptor::new("base::Service");
        inherited.queries.push(FunctionEntry {
            name: "Bar".to_owned(),
            entry_id: 2,
        });

        let set =
            InterfaceDescriptorSet::new(primary.clone()).with_inherited(vec![inherited.clone()]);
        let collected = set
            .iter()
            .map(|desc| desc.interface_path.clone())
            .collect::<Vec<_>>();
        assert_eq!(
            collected,
            vec!["primary".to_owned(), "base::Service".to_owned()]
        );
        assert_eq!(set.primary, primary);
        assert_eq!(set.inherited, vec![inherited]);
    }

    #[test]
    fn inherited_signatures_produce_ids() {
        let mut primary = InterfaceDescriptor::new("example::Dog");
        primary.commands.push(FunctionEntry {
            name: "Bark".to_owned(),
            entry_id: 1,
        });

        let mut mammal = InterfaceDescriptor::new("example::Mammal");
        mammal.queries.push(FunctionEntry {
            name: "AvgWeight".to_owned(),
            entry_id: 5,
        });

        let mut walker = InterfaceDescriptor::new("example::Walker");
        walker.commands.push(FunctionEntry {
            name: "Walk".to_owned(),
            entry_id: 3,
        });

        let set = InterfaceDescriptorSet::new(primary).with_inherited(vec![mammal, walker]);
        let ids = set.iter().map(super::compute_ids).collect::<Vec<_>>();
        assert_eq!(ids.len(), 3);
        assert!(ids.windows(2).all(|pair| pair[0] != pair[1]));
    }

    #[test]
    fn compute_ids_from_document_matches_bytes() {
        use crate::canonical::{
            CanonicalDocument, CanonicalFunction, CanonicalHashMeta, CanonicalService,
            CanonicalType, FunctionKind,
        };
        use std::collections::BTreeMap;

        let mut document = CanonicalDocument {
            canon_schema: crate::canonical::CANONICAL_SCHEMA.to_owned(),
            canon_version: crate::canonical::CANONICAL_VERSION.to_owned(),
            hash: CanonicalHashMeta {
                algo: crate::canonical::CANONICAL_HASH_ALGO.to_owned(),
                domain: INTERFACE_HASH_DOMAIN_STR.to_owned(),
            },
            services: BTreeMap::new(),
            types: BTreeMap::new(),
        };

        let service = CanonicalService {
            name: "example::Dog".to_owned(),
            extends: Vec::new(),
            functions: vec![CanonicalFunction {
                kind: FunctionKind::Command,
                name: "Bark".to_owned(),
                route: None,
                params: Vec::new(),
                returns: CanonicalType::Primitive {
                    name: "null".to_owned(),
                },
                entry_id_override: Some(1),
            }],
            events: Vec::new(),
        };

        document.services.insert(service.name.clone(), service);

        let id_doc = super::compute_ids_from_document(&document);
        let bytes = document.to_bytes().unwrap();
        let id_bytes = super::compute_ids_from_bytes(&bytes);
        assert_eq!(id_doc, id_bytes);
    }
}

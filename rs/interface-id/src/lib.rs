use blake3::{Hash, Hasher};
use serde::Serialize;
use serde_json::to_value;
use serde_json_canonicalizer::to_vec as to_canonical_vec;

pub mod canonical;
pub mod canonical_type;
pub mod runtime;

/// Domain separator for interface-level hashing.
pub const INTERFACE_HASH_DOMAIN: &[u8] = b"GEAR-IDL/v1:interface";
/// Domain separator for function/message hashing.
pub const FUNCTION_HASH_DOMAIN: &[u8] = b"GEAR-IDL/v1:func";

/// Canonical description of a Sails service used to derive stable interface identifiers.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct InterfaceSignature {
    pub interface_path: String,
    pub extends: Vec<ExtendedInterfaceSignature>,
    pub commands: Vec<FnSignature>,
    pub queries: Vec<FnSignature>,
    pub events: Vec<EventSignature>,
}

/// Canonical description of a command/query inside a service.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct FnSignature {
    pub name: String,
    pub opcode: u16,
}

/// Canonical description of an event emitted by a service.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct EventSignature {
    pub name: String,
    pub code: u16,
}

/// Canonical description of a parent interface implemented by a service.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ExtendedInterfaceSignature {
    pub name: String,
    pub interface_id32: u32,
    pub interface_uid64: u64,
}

/// Collection of interface signatures that represent a concrete service and
/// any interfaces it directly exposes via inheritance.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct InterfaceSignatureSet {
    pub primary: InterfaceSignature,
    #[serde(default)]
    pub inherited: Vec<InterfaceSignature>,
}

impl InterfaceSignature {
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

impl InterfaceSignatureSet {
    pub fn new(primary: InterfaceSignature) -> Self {
        Self {
            primary,
            inherited: Vec::new(),
        }
    }

    pub fn with_inherited(mut self, inherited: Vec<InterfaceSignature>) -> Self {
        self.inherited = inherited;
        self
    }

    pub fn iter(&self) -> impl Iterator<Item = &InterfaceSignature> {
        std::iter::once(&self.primary).chain(self.inherited.iter())
    }
}

/// Computes the `(interface_id32, interface_uid64)` pair using a canonical hash.
pub fn compute_ids(signature: &InterfaceSignature) -> (u32, u64) {
    let value =
        to_value(signature).expect("serializing interface signature to value should succeed");
    let canonical =
        to_canonical_vec(&value).expect("canonicalizing interface signature should succeed");
    compute_ids_from_bytes(&canonical)
}

/// Computes interface identifiers from a [`canonical::CanonicalDocument`].
pub fn compute_ids_from_document(doc: &canonical::CanonicalDocument) -> (u32, u64) {
    let bytes = doc
        .to_bytes()
        .expect("serializing canonical document should succeed");
    compute_ids_from_bytes(&bytes)
}

/// Computes interface identifiers from canonical bytes.
pub fn compute_ids_from_bytes(bytes: &[u8]) -> (u32, u64) {
    let digest = compute_interface_hash(bytes);
    let bytes = digest.as_bytes();
    let interface_id32 = u32::from_le_bytes(bytes[0..4].try_into().unwrap());
    let interface_uid64 = u64::from_le_bytes(bytes[0..8].try_into().unwrap());
    (interface_id32, interface_uid64)
}

/// Computes the full interface hash (BLAKE3-256) using the canonical domain separator.
pub fn compute_interface_hash(bytes: &[u8]) -> Hash {
    let mut hasher = Hasher::new();
    hasher.update(INTERFACE_HASH_DOMAIN);
    hasher.update(bytes);
    hasher.finalize()
}

/// Computes the 16-bit message identifier derived from a canonical function signature.
pub fn compute_msg_id16(
    interface_hash: &Hash,
    signature: &str,
    override_value: Option<u16>,
) -> u16 {
    if let Some(value) = override_value {
        return value;
    }

    let mut hasher = Hasher::new();
    hasher.update(FUNCTION_HASH_DOMAIN);
    hasher.update(interface_hash.as_bytes());
    hasher.update(signature.as_bytes());
    let digest = hasher.finalize();
    u16::from_le_bytes(digest.as_bytes()[0..2].try_into().unwrap())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_are_stable() {
        let mut signature = InterfaceSignature::new("example::Service");
        signature.extends.push(ExtendedInterfaceSignature {
            name: "ParentA".to_owned(),
            interface_id32: 0x1234_5678,
            interface_uid64: 0xabcdef01_2345_6789,
        });
        signature.commands.push(FnSignature {
            name: "DoSomething".to_owned(),
            opcode: 1,
        });
        signature.queries.push(FnSignature {
            name: "GetValue".to_owned(),
            opcode: 2,
        });
        signature.events.push(EventSignature {
            name: "Occurred".to_owned(),
            code: 1,
        });
        let (id32_a, uid64_a) = compute_ids(&signature);
        let (id32_b, uid64_b) = compute_ids(&signature);
        assert_eq!(id32_a, id32_b);
        assert_eq!(uid64_a, uid64_b);
        assert_ne!(id32_a, 0);
        assert_ne!(uid64_a, 0);
    }

    #[test]
    fn signature_set_iteration_includes_inherited() {
        let mut primary = InterfaceSignature::new("primary");
        primary.commands.push(FnSignature {
            name: "Foo".to_owned(),
            opcode: 1,
        });

        let mut inherited = InterfaceSignature::new("base::Service");
        inherited.queries.push(FnSignature {
            name: "Bar".to_owned(),
            opcode: 2,
        });

        let set =
            InterfaceSignatureSet::new(primary.clone()).with_inherited(vec![inherited.clone()]);
        let collected = set
            .iter()
            .map(|sig| sig.interface_path.clone())
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
        let mut primary = InterfaceSignature::new("example::Dog");
        primary.commands.push(FnSignature {
            name: "Bark".to_owned(),
            opcode: 1,
        });

        let mut mammal = InterfaceSignature::new("example::Mammal");
        mammal.queries.push(FnSignature {
            name: "AvgWeight".to_owned(),
            opcode: 5,
        });

        let mut walker = InterfaceSignature::new("example::Walker");
        walker.commands.push(FnSignature {
            name: "Walk".to_owned(),
            opcode: 3,
        });

        let set = InterfaceSignatureSet::new(primary).with_inherited(vec![mammal, walker]);
        let ids = set.iter().map(super::compute_ids).collect::<Vec<_>>();
        assert_eq!(ids.len(), 3);
        assert!(ids.windows(2).all(|pair| pair[0] != pair[1]));
    }

    #[test]
    fn compute_ids_from_document_matches_bytes() {
        use crate::canonical::{
            CanonicalDocument, CanonicalFunction, CanonicalService, CanonicalType, FunctionKind,
        };
        use std::collections::BTreeMap;

        let mut document = CanonicalDocument {
            version: crate::canonical::CANONICAL_VERSION.to_owned(),
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
                message_id_override: Some(1),
            }],
            events: Vec::new(),
        };

        document.services.insert(service.name.clone(), service);

        let (id32_doc, uid64_doc) = super::compute_ids_from_document(&document);
        let bytes = document.to_bytes().unwrap();
        let (id32_bytes, uid64_bytes) = super::compute_ids_from_bytes(&bytes);
        assert_eq!(id32_doc, id32_bytes);
        assert_eq!(uid64_doc, uid64_bytes);
    }
}

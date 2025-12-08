use crate::{
    Vec,
    meta::InterfaceId,
    scale_codec::{Decode, Encode, Error, Input, Output},
};

/// Sails protocol highest supported version.
pub const HIGHEST_SUPPORTED_VERSION: u8 = 1;

/// Sails protocol magic bytes.
/// Bytes stand for "GM" utf-8 string.
pub const MAGIC_BYTES: [u8; 2] = [0x47, 0x4D];

/// Minimal Sails message header length in bytes.
pub const MINIMAL_HLEN: u8 = 16;

/// Sails message header.
///
/// The header is a feature of an IDLv2. It gives opportunity for on-top of blockchain
/// services to trace sails messages and programs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SailsMessageHeader {
    version: Version,
    hlen: HeaderLength,
    interface_id: InterfaceId,
    route_id: u8,
    entry_id: u16,
}

impl SailsMessageHeader {
    /// Creates a new Sails message header.
    pub fn new(
        version: Version,
        hlen: HeaderLength,
        interface_id: InterfaceId,
        route_id: u8,
        entry_id: u16,
    ) -> Self {
        Self {
            version,
            hlen,
            interface_id,
            route_id,
            entry_id,
        }
    }

    /// Gets the version of the header.
    pub fn version(&self) -> Version {
        self.version
    }

    /// Gets the header length.
    pub fn hlen(&self) -> HeaderLength {
        self.hlen
    }

    /// Gets the interface ID.
    pub fn interface_id(&self) -> InterfaceId {
        self.interface_id
    }

    /// Gets the route ID.
    pub fn route_id(&self) -> u8 {
        self.route_id
    }

    /// Gets the entry ID.
    pub fn entry_id(&self) -> u16 {
        self.entry_id
    }
}

// Serialization and deserialization
impl SailsMessageHeader {
    /// Serialize header to bytes.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(self.hlen.inner() as usize);
        bytes.extend_from_slice(Magic::new().as_bytes());
        bytes.push(self.version.inner());
        bytes.push(self.hlen.inner());
        bytes.extend_from_slice(self.interface_id.as_bytes());
        bytes.extend_from_slice(&self.entry_id.to_le_bytes());
        bytes.push(self.route_id);
        // Reserved byte
        bytes.push(0);

        bytes
    }

    /// Deserialize header from bytes advancing the slice.
    pub fn try_read_bytes(bytes: &mut &[u8]) -> Result<Self, &'static str> {
        if bytes.len() < MINIMAL_HLEN as usize {
            return Err("Insufficient bytes for header");
        }

        // Validate and consume magic bytes.
        Magic::try_read_bytes(bytes)?;

        let version = Version::try_read_bytes(bytes)?;
        let hlen = HeaderLength::try_read_bytes(bytes)?;
        let interface_id = InterfaceId::try_read_bytes(bytes)?;

        let entry_id = u16::from_le_bytes([bytes[0], bytes[1]]);
        let route_id = bytes[2];
        let reserved = bytes[3];

        if version == Version::v1() && reserved != 0 {
            return Err("Reserved byte must be zero in version 1");
        }

        // Read 4 bytes for entry_id, route_id and reserved.
        *bytes = &bytes[4..];

        Ok(Self {
            version,
            hlen,
            interface_id,
            route_id,
            entry_id,
        })
    }

    /// Deserialize header from bytes (expects magic bytes at the start) without mutating the input.
    pub fn try_from_bytes(bytes: &[u8]) -> Result<Self, &'static str> {
        let mut slice = bytes;
        Self::try_read_bytes(&mut slice)
    }

    /// Tries to match the header's interface ID and route ID against a list of known interfaces in the program.
    pub fn try_match_interfaces(
        self,
        interfaces: &[(InterfaceId, u8)],
    ) -> Result<MatchedInterface, &'static str> {
        let Self {
            interface_id,
            route_id: message_route_id,
            entry_id,
            ..
        } = self;

        let (same_interface_ids, has_route) = interfaces
            .iter()
            .filter_map(|(id, r_id)| (*id == interface_id).then_some(*r_id))
            .fold((0, false), |(count, found), program_route_id| {
                let new_count = count + 1;

                let new_found = if !found {
                    message_route_id == program_route_id
                } else {
                    found
                };

                (new_count, new_found)
            });

        if same_interface_ids == 0 {
            Err("No matching interface ID found")
        } else if message_route_id == 0 && same_interface_ids > 1 {
            Err("Can't infer the interface by route id 0, many instances")
        } else if !has_route && message_route_id != 0 {
            // In case of route_id == 0, the has_route is always false
            Err("No matching route ID found for the interface ID")
        } else {
            Ok(MatchedInterface {
                interface_id,
                entry_id,
                route_id: message_route_id,
            })
        }
    }
}

impl Encode for SailsMessageHeader {
    fn encode_to<O: Output + ?Sized>(&self, dest: &mut O) {
        let bytes = self.to_bytes();
        dest.write(&bytes);
    }
}

impl Decode for SailsMessageHeader {
    fn decode<I: Input>(input: &mut I) -> Result<Self, Error> {
        let mut header_bytes = [0u8; MINIMAL_HLEN as usize]; // Include magic bytes
        input.read(&mut header_bytes)?;

        let mut slice = header_bytes.as_slice();
        Self::try_read_bytes(&mut slice).map_err(Error::from)
    }
}

/// Sails message header's protocol magic bytes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Encode)]
pub struct Magic([u8; 2]);

impl Magic {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self(MAGIC_BYTES)
    }

    /// Get magic bytes as a byte slice.
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    /// Deserialize from bytes, advancing the slice.
    pub fn try_read_bytes(bytes: &mut &[u8]) -> Result<Self, &'static str> {
        if bytes.len() < MAGIC_BYTES.len() {
            return Err("Insufficient bytes for magic");
        }

        let magic = [bytes[0], bytes[1]];
        if magic != MAGIC_BYTES {
            return Err("Invalid Sails magic bytes");
        }

        *bytes = &bytes[2..];
        Ok(Self(magic))
    }

    /// Deserialize from bytes without mutating the input.
    pub fn try_from_bytes(bytes: &[u8]) -> Result<Self, &'static str> {
        let mut slice = bytes;
        Self::try_read_bytes(&mut slice)
    }
}

impl Decode for Magic {
    fn decode<I: Input>(input: &mut I) -> Result<Self, Error> {
        let mut magic = [0u8; 2];
        input.read(&mut magic)?;

        let mut slice = magic.as_slice();
        Self::try_read_bytes(&mut slice).map_err(Error::from)
    }
}

/// Sails message header's protocol version.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Encode)]
pub struct Version(u8);

impl Version {
    /// Instantiates the type with version 1.
    pub fn v1() -> Self {
        Self(1)
    }

    /// Instantiates the type with the latest supported version.
    pub fn latest() -> Self {
        Self(HIGHEST_SUPPORTED_VERSION)
    }

    /// Creates a new version instance if the version is supported.
    ///
    /// Returns error if the version is unsupported, i.e. if:
    /// - version is 0
    /// - version is greater than highest supported version
    pub fn new(version: u8) -> Result<Self, &'static str> {
        if version == 0 || version > HIGHEST_SUPPORTED_VERSION {
            Err("Unsupported Sails version")
        } else {
            Ok(Self(version))
        }
    }

    /// Get inner version type.
    pub fn inner(&self) -> u8 {
        self.0
    }

    /// Deserialize from bytes, advancing the slice.
    pub fn try_read_bytes(bytes: &mut &[u8]) -> Result<Self, &'static str> {
        if bytes.is_empty() {
            return Err("Insufficient bytes for version");
        }

        let version = bytes[0];
        *bytes = &bytes[1..];
        Self::new(version)
    }

    /// Deserialize from bytes without mutating the input.
    pub fn try_from_bytes(bytes: &[u8]) -> Result<Self, &'static str> {
        let mut slice = bytes;
        Self::try_read_bytes(&mut slice)
    }
}

impl Decode for Version {
    fn decode<I: Input>(input: &mut I) -> Result<Self, Error> {
        let version = input.read_byte()?;
        let version_array = [version];

        Self::try_read_bytes(&mut version_array.as_slice()).map_err(Error::from)
    }
}

/// Sails message header length.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Encode)]
pub struct HeaderLength(u8);

impl HeaderLength {
    pub fn new(hlen: u8) -> Result<Self, &'static str> {
        if hlen < MINIMAL_HLEN {
            Err("Header length is less than minimal Sails header length")
        } else {
            Ok(Self(hlen))
        }
    }

    /// Get the header length as a u8.
    pub fn inner(&self) -> u8 {
        self.0
    }

    /// Deserialize from bytes, advancing the slice.
    pub fn try_read_bytes(bytes: &mut &[u8]) -> Result<Self, &'static str> {
        if bytes.is_empty() {
            return Err("Insufficient bytes for header length");
        }

        let hlen = bytes[0];
        *bytes = &bytes[1..];
        Self::new(hlen)
    }

    /// Deserialize from bytes without mutating the input.
    pub fn try_from_bytes(bytes: &[u8]) -> Result<Self, &'static str> {
        let mut slice = bytes;
        Self::try_read_bytes(&mut slice)
    }
}

impl Decode for HeaderLength {
    fn decode<I: Input>(input: &mut I) -> Result<Self, Error> {
        let hlen = input.read_byte()?;

        let hlen_array = [hlen];
        Self::try_read_bytes(&mut hlen_array.as_slice()).map_err(Error::from)
    }
}

/// The outcome of matching a message header against known interfaces.
///
/// Contains the matched interface ID, route ID, and entry ID to be executed.
///
/// The type is only instantiated upon successful matching. This guarantees, that
/// the contained values are against known interfaces map.
#[derive(Debug)]
pub struct MatchedInterface {
    interface_id: InterfaceId,
    route_id: u8,
    entry_id: u16,
}

impl MatchedInterface {
    /// Consumes the matched interface and returns its components.
    pub fn into_inner(self) -> (InterfaceId, u8, u16) {
        (self.interface_id, self.route_id, self.entry_id)
    }
}

/// Sails message wrapper that owns both header and payload.
///
/// This type is designed to be decoded from incoming messages and provides
/// convenient access to routing information while owning the payload data.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SailsMessage {
    header: SailsMessageHeader,
    payload: Vec<u8>,
}

impl SailsMessage {
    /// Creates a new Sails message form header and encodable payload type.
    pub fn new(header: SailsMessageHeader, payload: impl Encode) -> Self {
        Self {
            header,
            payload: payload.encode(),
        }
    }

    /// Gets a reference to the header.
    pub fn header(&self) -> &SailsMessageHeader {
        &self.header
    }

    /// Gets a reference to the payload.
    pub fn payload(&self) -> &[u8] {
        &self.payload
    }

    /// Matches the message header against known interfaces and returns routing information.
    ///
    /// Returns `(interface_id, route_id, entry_id, payload)` on success.
    pub fn try_match_interfaces(
        self,
        interfaces: &[(InterfaceId, u8)],
    ) -> Result<(InterfaceId, u8, u16, Vec<u8>), &'static str> {
        let matched = self.header.try_match_interfaces(interfaces)?;
        let (interface_id, route_id, entry_id) = matched.into_inner();
        Ok((interface_id, route_id, entry_id, self.payload))
    }
}

impl Decode for SailsMessage {
    fn decode<I: Input>(input: &mut I) -> Result<Self, Error> {
        // Decode the header
        let header = SailsMessageHeader::decode(input)?;
        let payload = Decode::decode(input)?;

        Ok(Self { header, payload })
    }
}

impl Encode for SailsMessage {
    fn encode_to<O: Output + ?Sized>(&self, dest: &mut O) {
        self.header.encode_to(dest);
        dest.write(&self.payload);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use scale_info::prelude::vec;

    #[test]
    fn try_from_bytes_does_not_move_offset() {
        let magic = Magic::new();
        let bytes = magic.as_bytes();

        let _ = Magic::try_from_bytes(bytes).expect("same bytes");
        assert_eq!(bytes, [0x47, 0x4D]);
    }

    #[test]
    fn magic_codec() {
        let magic = Magic::new();

        let encoded = magic.encode();
        assert_eq!(MAGIC_BYTES.encode(), encoded);
        let decoded = Magic::decode(&mut &encoded[..]).unwrap();

        assert_eq!(magic, decoded);
    }

    #[test]
    fn magic_serde() {
        let magic = Magic::new();

        let mut serialized = magic.as_bytes();
        assert_eq!(MAGIC_BYTES, serialized);
        let deserialized = Magic::try_read_bytes(&mut serialized).unwrap();

        assert_eq!(magic, deserialized);
    }

    #[test]
    fn magic_try_read_fails() {
        // Invalid bytes
        let invalid_bytes = [0x00, 0x00];
        let mut slice = invalid_bytes.as_slice();
        let result = Magic::try_read_bytes(&mut slice);

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Invalid Sails magic bytes");

        // Insufficient bytes
        let short_bytes = [0x47];
        let mut slice = short_bytes.as_slice();
        let result = Magic::try_read_bytes(&mut slice);

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Insufficient bytes for magic");
    }

    #[test]
    fn version_serde() {
        let version = Version::new(1).unwrap();

        let serialized = version.inner();
        assert_eq!(1u8, serialized);
        let deserialized = Version::try_read_bytes(&mut [serialized].as_slice()).unwrap();

        assert_eq!(version, deserialized);
    }

    #[test]
    fn version_codec() {
        let version = Version::new(1).unwrap();

        let encoded = version.encode();
        assert_eq!(1u8.encode(), encoded);
        let decoded = Version::decode(&mut &encoded[..]).unwrap();

        assert_eq!(version, decoded);
    }

    #[test]
    fn version_try_read_fails() {
        // Unsupported version
        let bytes1 = [255];
        let bytes2 = [0];
        let mut slice1 = bytes1.as_slice();
        let mut slice2 = bytes2.as_slice();

        let result1 = Version::try_read_bytes(&mut slice1);
        let result2 = Version::try_read_bytes(&mut slice2);
        assert!(result1.is_err());
        assert!(result2.is_err());

        assert_eq!(result1.unwrap_err(), "Unsupported Sails version");
        assert_eq!(result2.unwrap_err(), "Unsupported Sails version");

        // Insufficient bytes
        let bytes: [u8; 0] = [];
        let mut slice = bytes.as_slice();
        let result = Version::try_read_bytes(&mut slice);

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Insufficient bytes for version");
    }

    #[test]
    fn version_latest() {
        let version = Version::latest();
        assert_eq!(version.inner(), HIGHEST_SUPPORTED_VERSION);
    }

    #[test]
    fn header_length_serde() {
        let hlen = HeaderLength::new(20).unwrap();

        let serialized = hlen.inner();
        assert_eq!(20u8, serialized);
        let deserialized = HeaderLength::try_read_bytes(&mut [serialized].as_slice()).unwrap();

        assert_eq!(hlen, deserialized);
    }

    #[test]
    fn header_length_codec() {
        let hlen = HeaderLength::new(20).unwrap();

        let encoded = hlen.encode();
        assert_eq!(20u8.encode(), encoded);
        let decoded = HeaderLength::decode(&mut &encoded[..]).unwrap();

        assert_eq!(hlen, decoded);
    }

    #[test]
    fn header_try_read_fails() {
        // Header length less than minimal
        let bytes1 = [MINIMAL_HLEN - 1];
        let mut slice1 = bytes1.as_slice();
        let result1 = HeaderLength::try_read_bytes(&mut slice1);

        assert!(result1.is_err());
        assert_eq!(
            result1.unwrap_err(),
            "Header length is less than minimal Sails header length"
        );

        // Insufficient bytes
        let bytes: [u8; 0] = [];
        let mut slice = bytes.as_slice();
        let result2 = HeaderLength::try_read_bytes(&mut slice);

        assert!(result2.is_err());
        assert_eq!(result2.unwrap_err(), "Insufficient bytes for header length");
    }

    #[test]
    fn message_header_serde() {
        let header = SailsMessageHeader {
            version: Version::new(1).unwrap(),
            hlen: HeaderLength::new(MINIMAL_HLEN).unwrap(),
            interface_id: InterfaceId([1, 2, 3, 4, 5, 6, 7, 8]),
            route_id: 42,
            entry_id: 1234,
        };

        let bytes = header.to_bytes();
        assert_eq!(bytes.len(), MINIMAL_HLEN as usize);
        assert_eq!(
            bytes,
            vec![
                0x47, 0x4D, // magic ("GM")
                1,    // version
                16,   // hlen
                1, 2, 3, 4, 5, 6, 7, 8, // interface_id
                210, 4,  // entry_id (1234 in little-endian)
                42, // route_id
                0,  // reserved
            ]
        );

        let mut slice = bytes.as_slice();
        let deserialized = SailsMessageHeader::try_read_bytes(&mut slice).unwrap();

        assert_eq!(header, deserialized);
        assert_eq!(slice.len(), 0); // all bytes consumed
    }

    #[test]
    fn message_header_try_read_fails_invalid_magic() {
        // Insufficient bytes (no route id)
        let bytes = [0x47, 0x4D, 1, 15, 1, 2, 3, 4, 5, 6, 7, 8, 210, 4];

        let mut slice = bytes.as_slice();
        let result = SailsMessageHeader::try_read_bytes(&mut slice);

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Insufficient bytes for header");
    }

    #[test]
    fn message_header_serde_with_surplus() {
        let header_bytes = vec![
            0x47, 0x4D, // magic ("GM")
            1,    // version
            16,   // hlen
            1, 2, 3, 4, 5, 6, 7, 8, // interface_id
            210, 4,  // entry_id (1234 in little-endian)
            42, // route_id
            0,  // reserved
            // Surplus bytes (payload)
            99, 100, 101,
        ];
        let mut slice = header_bytes.as_slice();
        let deserialized = SailsMessageHeader::try_read_bytes(&mut slice).unwrap();
        assert_eq!(deserialized.version, Version::new(1).unwrap());
        assert_eq!(deserialized.hlen, HeaderLength::new(MINIMAL_HLEN).unwrap());
        assert_eq!(
            deserialized.interface_id,
            InterfaceId([1, 2, 3, 4, 5, 6, 7, 8])
        );
        assert_eq!(deserialized.entry_id, 1234);
        assert_eq!(deserialized.route_id, 42);
        assert_eq!(slice, &[99, 100, 101]); // Surplus bytes
    }

    #[test]
    fn message_header_with_non_zero_reserved_fails() {
        // Reserved byte is non-zero when version is 1
        let header_bytes = vec![
            0x47, 0x4D, // magic ("GM")
            1,    // version
            16,   // hlen
            1, 2, 3, 4, 5, 6, 7, 8, // interface_id
            210, 4,  // entry_id (1234 in little-endian)
            42, // route_id
            1,  // reserved (non-zero)
        ];
        let mut slice = header_bytes.as_slice();
        let result = SailsMessageHeader::try_read_bytes(&mut slice);

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "Reserved byte must be zero in version 1"
        );

        // Reserved byte is non-zero when version is 2 (unsupported currently version)
        let header_bytes = vec![
            0x47, 0x4D, // magic ("GM")
            2,    // version
            16,   // hlen
            1, 2, 3, 4, 5, 6, 7, 8, // interface_id
            210, 4,  // entry_id (1234 in little-endian)
            42, // route_id
            1,  // reserved (non-zero)
        ];
        let mut slice = header_bytes.as_slice();
        let result = SailsMessageHeader::try_read_bytes(&mut slice);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Unsupported Sails version");
    }

    #[test]
    fn match_interfaces_works() {
        // Simple test case
        let header = SailsMessageHeader {
            version: Version::new(1).unwrap(),
            hlen: HeaderLength::new(16).unwrap(),
            interface_id: InterfaceId([1, 2, 3, 4, 5, 6, 7, 8]),
            route_id: 1,
            entry_id: 100,
        };

        let interfaces = [(InterfaceId([1, 2, 3, 4, 5, 6, 7, 8]), 1)];
        let result = header.try_match_interfaces(&interfaces).unwrap();
        let (iid, rid, eid) = result.into_inner();

        assert_eq!(iid, InterfaceId([1, 2, 3, 4, 5, 6, 7, 8]));
        assert_eq!(rid, 1);
        assert_eq!(eid, 100);

        // Route id zero with single matching interface
        let header = SailsMessageHeader {
            version: Version::new(1).unwrap(),
            hlen: HeaderLength::new(16).unwrap(),
            interface_id: InterfaceId([9, 8, 7, 6, 5, 4, 3, 2]),
            route_id: 0,
            entry_id: 200,
        };

        let interfaces = [
            (InterfaceId([9, 8, 7, 6, 5, 4, 3, 2]), 42),
            (InterfaceId([1, 2, 3, 4, 5, 6, 7, 8]), 42),
        ];

        let result = header.try_match_interfaces(&interfaces).unwrap();
        let (iid, rid, eid) = result.into_inner();
        assert_eq!(iid, InterfaceId([9, 8, 7, 6, 5, 4, 3, 2]));
        assert_eq!(rid, 0);
        assert_eq!(eid, 200);
    }

    #[test]
    fn match_interfaces_no_match() {
        let header = SailsMessageHeader {
            version: Version::new(1).unwrap(),
            hlen: HeaderLength::new(16).unwrap(),
            interface_id: InterfaceId([1, 2, 3, 4, 5, 6, 7, 8]),
            route_id: 1,
            entry_id: 100,
        };

        let interfaces = [(InterfaceId([9, 9, 9, 9, 9, 9, 9, 9]), 1)];
        let result = header.try_match_interfaces(&interfaces);

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "No matching interface ID found");
    }

    #[test]
    fn match_interfaces_multiple_same_interface_with_route_zero() {
        let header = SailsMessageHeader {
            version: Version::new(1).unwrap(),
            hlen: HeaderLength::new(16).unwrap(),
            interface_id: InterfaceId([1, 2, 3, 4, 5, 6, 7, 8]),
            route_id: 0,
            entry_id: 100,
        };

        let interfaces = [
            (InterfaceId([1, 2, 3, 4, 5, 6, 7, 8]), 1),
            (InterfaceId([1, 2, 3, 4, 5, 6, 7, 8]), 2),
        ];
        let result = header.try_match_interfaces(&interfaces);

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "Can't infer the interface by route id 0, many instances"
        );
    }

    #[test]
    fn match_interfaces_route_mismatch() {
        let header = SailsMessageHeader {
            version: Version::new(1).unwrap(),
            hlen: HeaderLength::new(16).unwrap(),
            interface_id: InterfaceId([1, 2, 3, 4, 5, 6, 7, 8]),
            route_id: 5,
            entry_id: 100,
        };

        let interfaces = [(InterfaceId([1, 2, 3, 4, 5, 6, 7, 8]), 1)];
        let result = header.try_match_interfaces(&interfaces);

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "No matching route ID found for the interface ID"
        );
    }
}

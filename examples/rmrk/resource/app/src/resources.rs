use gstd::{prelude::*, ActorId};

pub type PartId = u32;

pub type ResourceId = u8;

#[derive(Decode, Encode, TypeInfo, Clone, Debug)]
pub enum Resource {
    Basic(BasicResource),
    Slot(SlotResource),
    Composed(ComposedResource),
}

#[derive(Decode, Encode, TypeInfo, Clone, Debug)]
pub struct BasicResource {
    /// URI like IPFS hash
    pub src: String,

    /// If the resource has the thumb property, this will be a URI to a thumbnail of the given
    /// resource.
    pub thumb: Option<String>,

    /// Reference to IPFS location of metadata
    pub metadata_uri: String,
}

#[derive(Decode, Encode, TypeInfo, Clone, Debug)]
pub struct ComposedResource {
    /// URI like ipfs hash
    pub src: String,

    /// If the resource has the thumb property, this will be a URI to a thumbnail of the given
    /// resource.
    pub thumb: String,

    /// Reference to IPFS location of metadata
    pub metadata_uri: String,

    // The address of base contract
    pub base: ActorId,

    //  If a resource is composed, it will have an array of parts that compose it
    pub parts: Vec<PartId>,
}

#[derive(Decode, Encode, TypeInfo, Clone, Debug)]
pub struct SlotResource {
    /// URI like ipfs hash
    pub src: String,

    /// If the resource has the thumb property, this will be a URI to a thumbnail of the given
    /// resource.
    pub thumb: String,

    /// Reference to IPFS location of metadata
    pub metadata_uri: String,

    // The address of base contract
    pub base: ActorId,

    /// If the resource has the slot property, it was designed to fit into a specific Base's slot.
    pub slot: PartId,
}

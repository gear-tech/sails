use sails_rs::prelude::*;

pub type CollectionId = ActorId;
pub type ZIndex = u32;

pub type PartId = u32;

#[derive(Decode, Encode, TypeInfo, Clone)]
pub enum Part {
    Fixed(FixedPart),
    Slot(SlotPart),
}

#[derive(Decode, Encode, TypeInfo, Clone)]
pub struct FixedPart {
    /// An optional zIndex of base part layer.
    /// specifies the stack order of an element.
    /// An element with greater stack order is always in front of an element with a lower stack order.
    pub z: Option<ZIndex>,

    /// The metadata URI of the part.
    pub metadata_uri: String,
}

#[derive(Decode, Encode, TypeInfo, Clone)]
pub struct SlotPart {
    /// Array of whitelisted collections that can be equipped in the given slot. Used with slot parts only.
    pub equippable: Vec<CollectionId>,

    /// An optional zIndex of base part layer.
    /// specifies the stack order of an element.
    /// An element with greater stack order is always in front of an element with a lower stack order.
    pub z: Option<ZIndex>,

    /// The metadata URI of the part.
    pub metadata_uri: String,
}

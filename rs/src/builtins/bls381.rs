use crate::{
    ActorId,
    builtins::{BuiltinsRemoting, builtin_action},
    calls::{ActionIo, Call, RemotingAction},
    errors::{Error, Result},
    prelude::{Decode, Encode, TypeInfo, Vec},
};
use gbuiltin_bls381::{Request as GearBls381Request, Response as GearBls381Response};

/// Gear protocol BLS381 builtin id is 0x6b6e292c382945e80bf51af2ba7fe9f458dcff81ae6075c46f9095e1bbecdc37
pub const BLS381_BUILTIN_ID: ActorId = ActorId::new([
    0x6b, 0x6e, 0x29, 0x2c, 0x38, 0x29, 0x45, 0xe8, 0x0b, 0xf5, 0x1a, 0xf2, 0xba, 0x7f, 0xe9, 0xf4,
    0x58, 0xdc, 0xff, 0x81, 0xae, 0x60, 0x75, 0xc4, 0x6f, 0x90, 0x95, 0xe1, 0xbb, 0xec, 0xdc, 0x37,
]);

builtin_action!(
    Bls381Request,
    Bls381Builtin,
    MultiMillerLoop { a: Vec<u8>, b: Vec<u8> } => Bls381Response
);

builtin_action!(
    Bls381Request,
    Bls381Builtin,
    FinalExponentiation { f: Vec<u8> } => Bls381Response
);

builtin_action!(
    Bls381Request,
    Bls381Builtin,
    MultiScalarMultiplicationG1 { bases: Vec<u8>, scalars: Vec<u8> } => Bls381Response
);

builtin_action!(
    Bls381Request,
    Bls381Builtin,
    MultiScalarMultiplicationG2 { bases: Vec<u8>, scalars: Vec<u8> } => Bls381Response
);

builtin_action!(
    Bls381Request,
    Bls381Builtin,
    ProjectiveMultiplicationG1 { base: Vec<u8>, scalar: Vec<u8> } => Bls381Response
);

builtin_action!(
    Bls381Request,
    Bls381Builtin,
    ProjectiveMultiplicationG2 { base: Vec<u8>, scalar: Vec<u8> } => Bls381Response
);

builtin_action!(
    Bls381Request,
    Bls381Builtin,
    AggregateG1 { points: Vec<u8> } => Bls381Response
);

builtin_action!(
    Bls381Request,
    Bls381Builtin,
    MapToG2Affine { message: Vec<u8> } => Bls381Response
);

pub struct Bls381Builtin<R> {
    remoting: R,
}

impl<R: BuiltinsRemoting + Clone> Bls381Builtin<R> {
    pub fn new(remoting: R) -> Self {
        Self { remoting }
    }
}

/// `TypeInfo` implementor copy of `gbuiltin_bls381::Request`.
#[derive(Debug, Clone, Eq, PartialEq, Encode, Decode, TypeInfo)]
pub enum Bls381Request {
    /// Request to pairing multi Miller loop for *BLS12-381*.
    ///
    /// Encoded:
    ///   - `a`: [`ArkScale<Vec<G1Affine>>`].
    ///   - `b`: [`ArkScale<Vec<G2Affine>>`].
    #[codec(index = 0)]
    MultiMillerLoop { a: Vec<u8>, b: Vec<u8> },

    /// Request to pairing final exponentiation for *BLS12-381*.
    ///
    /// Encoded: [`ArkScale<<Bls12_381::TargetField>`].
    #[codec(index = 1)]
    FinalExponentiation { f: Vec<u8> },

    /// Request to multi scalar multiplication on *G1* for *BLS12-381*
    ///
    /// Encoded:
    ///   - `bases`: [`ArkScale<Vec<G1Affine>>`].
    ///   - `scalars`: [`ArkScale<Vec<G1Config::ScalarField>>`].
    #[codec(index = 2)]
    MultiScalarMultiplicationG1 { bases: Vec<u8>, scalars: Vec<u8> },

    /// Request to multi scalar multiplication on *G2* for *BLS12-381*
    ///
    /// Encoded:
    ///   - `bases`: [`ArkScale<Vec<G2Affine>>`].
    ///   - `scalars`: [`ArkScale<Vec<G2Config::ScalarField>>`].
    #[codec(index = 3)]
    MultiScalarMultiplicationG2 { bases: Vec<u8>, scalars: Vec<u8> },

    /// Request to projective multiplication on *G1* for *BLS12-381*.
    ///
    /// Encoded:
    ///   - `base`: [`ArkScaleProjective<G1Projective>`].
    ///   - `scalar`: [`ArkScale<Vec<u64>>`].
    #[codec(index = 4)]
    ProjectiveMultiplicationG1 { base: Vec<u8>, scalar: Vec<u8> },

    /// Request to projective multiplication on *G2* for *BLS12-381*.
    ///
    /// Encoded:
    ///   - `base`: [`ArkScaleProjective<G2Projective>`].
    ///   - `scalar`: [`ArkScale<Vec<u64>>`].
    #[codec(index = 5)]
    ProjectiveMultiplicationG2 { base: Vec<u8>, scalar: Vec<u8> },

    /// Request to aggregate *G1* points for *BLS12-381*.
    ///
    /// Encoded: [`ArkScale<Vec<G1Projective>>`].
    #[codec(index = 6)]
    AggregateG1 { points: Vec<u8> },

    /// Request to map an arbitrary message to *G2Affine* point for *BLS12-381*.
    ///
    /// Raw message bytes to map.
    #[codec(index = 7)]
    MapToG2Affine { message: Vec<u8> },
}

impl From<GearBls381Request> for Bls381Request {
    fn from(request: GearBls381Request) -> Self {
        match request {
            GearBls381Request::MultiMillerLoop { a, b } => Self::MultiMillerLoop { a, b },
            GearBls381Request::FinalExponentiation { f } => Self::FinalExponentiation { f },
            GearBls381Request::MultiScalarMultiplicationG1 { bases, scalars } => {
                Self::MultiScalarMultiplicationG1 { bases, scalars }
            }
            GearBls381Request::MultiScalarMultiplicationG2 { bases, scalars } => {
                Self::MultiScalarMultiplicationG2 { bases, scalars }
            }
            GearBls381Request::ProjectiveMultiplicationG1 { base, scalar } => {
                Self::ProjectiveMultiplicationG1 { base, scalar }
            }
            GearBls381Request::ProjectiveMultiplicationG2 { base, scalar } => {
                Self::ProjectiveMultiplicationG2 { base, scalar }
            }
            GearBls381Request::AggregateG1 { points } => Self::AggregateG1 { points },
            GearBls381Request::MapToG2Affine { message } => Self::MapToG2Affine { message },
        }
    }
}

/// `TypeInfo` implementor copy of `gbuiltin_bls381::Response`.
#[derive(Debug, Clone, Eq, PartialEq, Encode, Decode, TypeInfo)]
pub enum Bls381Response {
    /// Result of the multi Miller loop, encoded: [`ArkScale<Bls12_381::TargetField>`].
    #[codec(index = 0)]
    MultiMillerLoop(Vec<u8>),
    /// Result of the final exponentiation, encoded: [`ArkScale<Bls12_381::TargetField>`].
    #[codec(index = 1)]
    FinalExponentiation(Vec<u8>),
    /// Result of the multi scalar multiplication, encoded: [`ArkScaleProjective<G1Projective>`].
    #[codec(index = 2)]
    MultiScalarMultiplicationG1(Vec<u8>),
    /// Result of the multi scalar multiplication, encoded: [`ArkScaleProjective<G2Projective>`].
    #[codec(index = 3)]
    MultiScalarMultiplicationG2(Vec<u8>),
    /// Result of the projective multiplication, encoded: [`ArkScaleProjective<G1Projective>`].
    #[codec(index = 4)]
    ProjectiveMultiplicationG1(Vec<u8>),
    /// Result of the projective multiplication, encoded: [`ArkScaleProjective<G2Projective>`].
    #[codec(index = 5)]
    ProjectiveMultiplicationG2(Vec<u8>),
    /// Result of the aggregation, encoded: [`ArkScale<G1Projective>`].
    #[codec(index = 6)]
    AggregateG1(Vec<u8>),
    /// Result of the mapping, encoded: [`ArkScale<G2Affine>`].
    #[codec(index = 7)]
    MapToG2Affine(Vec<u8>),
}

impl From<GearBls381Response> for Bls381Response {
    fn from(response: GearBls381Response) -> Self {
        match response {
            GearBls381Response::MultiMillerLoop(value) => Self::MultiMillerLoop(value),
            GearBls381Response::FinalExponentiation(value) => Self::FinalExponentiation(value),
            GearBls381Response::MultiScalarMultiplicationG1(value) => {
                Self::MultiScalarMultiplicationG1(value)
            }
            GearBls381Response::MultiScalarMultiplicationG2(value) => {
                Self::MultiScalarMultiplicationG2(value)
            }
            GearBls381Response::ProjectiveMultiplicationG1(value) => {
                Self::ProjectiveMultiplicationG1(value)
            }
            GearBls381Response::ProjectiveMultiplicationG2(value) => {
                Self::ProjectiveMultiplicationG2(value)
            }
            GearBls381Response::AggregateG1(value) => Self::AggregateG1(value),
            GearBls381Response::MapToG2Affine(value) => Self::MapToG2Affine(value),
        }
    }
}

#[test]
fn test_id() {
    let expected = hex::decode("6b6e292c382945e80bf51af2ba7fe9f458dcff81ae6075c46f9095e1bbecdc37")
        .expect("Failed to decode hex");
    assert_eq!(BLS381_BUILTIN_ID.into_bytes().to_vec(), expected);
}

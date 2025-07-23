use crate::{
    builtins::{BuiltinsRemoting, builtin_action},
    calls::{ActionIo, Call, RemotingAction},
    errors::{Error, Result},
    prelude::{Decode, Encode, TypeInfo, Vec},
};
use gbuiltin_bls381::{Request as GearBls381Request, Response as GearBls381Response};

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{builtins::test_utils::assert_action_codec, prelude::vec};

    #[test]
    fn test_codec() {
        assert_action_codec!(
            Bls381Request,
            MultiMillerLoop {
                a: vec![1, 2, 3],
                b: vec![4, 5, 6]
            },
            Bls381Response,
            MultiMillerLoop(vec![7, 8, 9])
        );

        assert_action_codec!(
            Bls381Request,
            FinalExponentiation {
                f: vec![10, 11, 12]
            },
            Bls381Response,
            FinalExponentiation(vec![13, 14, 15])
        );

        assert_action_codec!(
            Bls381Request,
            MultiScalarMultiplicationG1 {
                bases: vec![16, 17, 18],
                scalars: vec![19, 20, 21]
            },
            Bls381Response,
            MultiScalarMultiplicationG1(vec![22, 23, 24])
        );

        assert_action_codec!(
            Bls381Request,
            MultiScalarMultiplicationG2 {
                bases: vec![25, 26, 27],
                scalars: vec![28, 29, 30]
            },
            Bls381Response,
            MultiScalarMultiplicationG2(vec![31, 32, 33])
        );

        assert_action_codec!(
            Bls381Request,
            ProjectiveMultiplicationG1 {
                base: vec![34, 35, 36],
                scalar: vec![37, 38, 39]
            },
            Bls381Response,
            ProjectiveMultiplicationG1(vec![40, 41, 42])
        );

        assert_action_codec!(
            Bls381Request,
            ProjectiveMultiplicationG2 {
                base: vec![43, 44, 45],
                scalar: vec![46, 47, 48]
            },
            Bls381Response,
            ProjectiveMultiplicationG2(vec![49, 50, 51])
        );

        assert_action_codec!(
            Bls381Request,
            AggregateG1 {
                points: vec![52, 53, 54]
            },
            Bls381Response,
            AggregateG1(vec![55, 56, 57])
        );

        assert_action_codec!(
            Bls381Request,
            MapToG2Affine {
                message: vec![58, 59, 60]
            },
            Bls381Response,
            MapToG2Affine(vec![61, 62, 63])
        );
    }
}

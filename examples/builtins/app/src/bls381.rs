use super::*;

pub struct Bls381Broker;

// Requests
// /// Request to pairing multi Miller loop for *BLS12-381*.
// ///
// /// Encoded:
// ///   - `a`: [`ArkScale<Vec<G1Affine>>`].
// ///   - `b`: [`ArkScale<Vec<G2Affine>>`].
// #[codec(index = 0)]
// MultiMillerLoop { a: Vec<u8>, b: Vec<u8> },

// /// Request to pairing final exponentiation for *BLS12-381*.
// ///
// /// Encoded: [`ArkScale<<Bls12_381::TargetField>`].
// #[codec(index = 1)]
// FinalExponentiation { f: Vec<u8> },

// /// Request to multi scalar multiplication on *G1* for *BLS12-381*
// ///
// /// Encoded:
// ///   - `bases`: [`ArkScale<Vec<G1Affine>>`].
// ///   - `scalars`: [`ArkScale<Vec<G1Config::ScalarField>>`].
// #[codec(index = 2)]
// MultiScalarMultiplicationG1 { bases: Vec<u8>, scalars: Vec<u8> },

// /// Request to multi scalar multiplication on *G2* for *BLS12-381*
// ///
// /// Encoded:
// ///   - `bases`: [`ArkScale<Vec<G2Affine>>`].
// ///   - `scalars`: [`ArkScale<Vec<G2Config::ScalarField>>`].
// #[codec(index = 3)]
// MultiScalarMultiplicationG2 { bases: Vec<u8>, scalars: Vec<u8> },

// /// Request to projective multiplication on *G1* for *BLS12-381*.
// ///
// /// Encoded:
// ///   - `base`: [`ArkScaleProjective<G1Projective>`].
// ///   - `scalar`: [`ArkScale<Vec<u64>>`].
// #[codec(index = 4)]
// ProjectiveMultiplicationG1 { base: Vec<u8>, scalar: Vec<u8> },

// /// Request to projective multiplication on *G2* for *BLS12-381*.
// ///
// /// Encoded:
// ///   - `base`: [`ArkScaleProjective<G2Projective>`].
// ///   - `scalar`: [`ArkScale<Vec<u64>>`].
// #[codec(index = 5)]
// ProjectiveMultiplicationG2 { base: Vec<u8>, scalar: Vec<u8> },

// /// Request to aggregate *G1* points for *BLS12-381*.
// ///
// /// Encoded: [`ArkScale<Vec<G1Projective>>`].
// #[codec(index = 6)]
// AggregateG1 { points: Vec<u8> },

// /// Request to map an arbitrary message to *G2Affine* point for *BLS12-381*.
// ///
// /// Raw message bytes to map.
// #[codec(index = 7)]
// MapToG2Affine { message: Vec<u8> },

#[sails_rs::service]
impl Bls381Broker {
    #[export]
    pub async fn multi_miller_loop(
        &mut self,
        a: Vec<u8>,
        b: Vec<u8>,
    ) -> Result<Bls381Response, String> {
        let bls381_builtin_client = Bls381Builtin::new(GStdRemoting::new());

        bls381_builtin_client
            .multi_miller_loop(a, b)
            .send_recv(BLS381_BUILTIN_ID)
            .await
            .map_err(|e| format!("failed sending bls381 builtin request: {e}"))
    }

    #[export]
    pub async fn final_exponentiation(&mut self, f: Vec<u8>) -> Result<Bls381Response, String> {
        let bls381_builtin_client = Bls381Builtin::new(GStdRemoting::new());

        bls381_builtin_client
            .final_exponentiation(f)
            .send_recv(BLS381_BUILTIN_ID)
            .await
            .map_err(|e| format!("failed sending bls381 builtin request: {e}"))
    }

    #[export]
    pub async fn multi_scalar_multiplication_g1(
        &mut self,
        bases: Vec<u8>,
        scalars: Vec<u8>,
    ) -> Result<Bls381Response, String> {
        let bls381_builtin_client = Bls381Builtin::new(GStdRemoting::new());

        bls381_builtin_client
            .multi_scalar_multiplication_g1(bases, scalars)
            .send_recv(BLS381_BUILTIN_ID)
            .await
            .map_err(|e| format!("failed sending bls381 builtin request: {e}"))
    }

    #[export]
    pub async fn multi_scalar_multiplication_g2(
        &mut self,
        bases: Vec<u8>,
        scalars: Vec<u8>,
    ) -> Result<Bls381Response, String> {
        let bls381_builtin_client = Bls381Builtin::new(GStdRemoting::new());

        bls381_builtin_client
            .multi_scalar_multiplication_g2(bases, scalars)
            .send_recv(BLS381_BUILTIN_ID)
            .await
            .map_err(|e| format!("failed sending bls381 builtin request: {e}"))
    }

    #[export]
    pub async fn projective_multiplication_g1(
        &mut self,
        base: Vec<u8>,
        scalar: Vec<u8>,
    ) -> Result<Bls381Response, String> {
        let bls381_builtin_client = Bls381Builtin::new(GStdRemoting::new());

        bls381_builtin_client
            .projective_multiplication_g1(base, scalar)
            .send_recv(BLS381_BUILTIN_ID)
            .await
            .map_err(|e| format!("failed sending bls381 builtin request: {e}"))
    }

    #[export]
    pub async fn projective_multiplication_g2(
        &mut self,
        base: Vec<u8>,
        scalar: Vec<u8>,
    ) -> Result<Bls381Response, String> {
        let bls381_builtin_client = Bls381Builtin::new(GStdRemoting::new());

        bls381_builtin_client
            .projective_multiplication_g2(base, scalar)
            .send_recv(BLS381_BUILTIN_ID)
            .await
            .map_err(|e| format!("failed sending bls381 builtin request: {e}"))
    }

    #[export]
    pub async fn aggregate_g1(&mut self, points: Vec<u8>) -> Result<Bls381Response, String> {
        let bls381_builtin_client = Bls381Builtin::new(GStdRemoting::new());

        bls381_builtin_client
            .aggregate_g1(points)
            .send_recv(BLS381_BUILTIN_ID)
            .await
            .map_err(|e| format!("failed sending bls381 builtin request: {e}"))
    }

    #[export]
    pub async fn map_to_g2_affine(&mut self, message: Vec<u8>) -> Result<Bls381Response, String> {
        let bls381_builtin_client = Bls381Builtin::new(GStdRemoting::new());

        bls381_builtin_client
            .map_to_g2_affine(message)
            .send_recv(BLS381_BUILTIN_ID)
            .await
            .map_err(|e| format!("failed sending bls381 builtin request: {e}"))
    }
}

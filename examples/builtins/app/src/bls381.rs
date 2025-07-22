use super::*;

pub struct Bls381Broker;

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

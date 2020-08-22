use super::{
    circuit::{Circuit, ConstraintSystem, MetaCircuit, Wire},
    domain::EvaluationDomain,
    Error, GATE_DEGREE, SRS,
};
use crate::arithmetic::{Curve, CurveAffine, Field};
use crate::polycommit::Params;

impl<C: CurveAffine> SRS<C> {
    /// This generates a structured reference string for the provided `circuit`
    /// and `params`.
    pub fn generate<ConcreteCircuit: Circuit<C::Scalar>>(
        params: &Params<C>,
        circuit: &ConcreteCircuit,
    ) -> Result<Self, Error> {
        struct Assembly<F: Field> {
            sa: Vec<F>,
            sb: Vec<F>,
            sc: Vec<F>,
            sd: Vec<F>,
            sm: Vec<F>,
        }

        impl<F: Field> ConstraintSystem<F> for Assembly<F> {
            fn create_gate(
                &mut self,
                sa: F,
                sb: F,
                sc: F,
                sd: F,
                sm: F,
                _: impl Fn() -> Result<(F, F, F, F), Error>,
            ) -> Result<(Wire, Wire, Wire, Wire), Error> {
                let tmp = Ok((
                    Wire::A(self.sa.len()),
                    Wire::B(self.sa.len()),
                    Wire::C(self.sa.len()),
                    Wire::D(self.sa.len()),
                ));
                self.sa.push(sa);
                self.sb.push(sb);
                self.sc.push(sc);
                self.sd.push(sd);
                self.sm.push(sm);
                tmp
            }
        }

        let mut assembly: Assembly<C::Scalar> = Assembly {
            sa: vec![],
            sb: vec![],
            sc: vec![],
            sd: vec![],
            sm: vec![],
        };

        let mut meta = MetaCircuit::default();

        let config = ConcreteCircuit::configure(&mut meta);

        // Synthesize the circuit to obtain SRS
        circuit.synthesize(&mut assembly, config)?;

        assembly.sa.resize(params.n as usize, C::Scalar::zero());
        assembly.sb.resize(params.n as usize, C::Scalar::zero());
        assembly.sc.resize(params.n as usize, C::Scalar::zero());
        assembly.sd.resize(params.n as usize, C::Scalar::zero());
        assembly.sm.resize(params.n as usize, C::Scalar::zero());

        // Compute commitments to the fixed wire values
        let sa_commitment = params
            .commit_lagrange(&assembly.sa, C::Scalar::one())
            .to_affine();
        let sb_commitment = params
            .commit_lagrange(&assembly.sb, C::Scalar::one())
            .to_affine();
        let sc_commitment = params
            .commit_lagrange(&assembly.sc, C::Scalar::one())
            .to_affine();
        let sd_commitment = params
            .commit_lagrange(&assembly.sd, C::Scalar::one())
            .to_affine();
        let sm_commitment = params
            .commit_lagrange(&assembly.sm, C::Scalar::one())
            .to_affine();

        let domain = EvaluationDomain::new(GATE_DEGREE, params.k);

        let sa = domain.obtain_coset(assembly.sa);
        let sb = domain.obtain_coset(assembly.sb);
        let sc = domain.obtain_coset(assembly.sc);
        let sd = domain.obtain_coset(assembly.sd);
        let sm = domain.obtain_coset(assembly.sm);

        Ok(SRS {
            sa,
            sb,
            sc,
            sd,
            sm,
            sa_commitment,
            sb_commitment,
            sc_commitment,
            sd_commitment,
            sm_commitment,
            domain,
        })
    }
}

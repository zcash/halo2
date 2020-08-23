use super::{
    circuit::{AdviceWire, Circuit, ConstraintSystem, FixedWire, MetaCircuit, Variable, Wire},
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
            fixed: Vec<Vec<F>>,
        }

        impl<F: Field> ConstraintSystem<F> for Assembly<F> {
            fn assign_advice(
                &mut self,
                _: AdviceWire,
                _: usize,
                _: impl FnOnce() -> Result<F, Error>,
            ) -> Result<(), Error> {
                // We only care about fixed wires here
                Ok(())
            }

            fn assign_fixed(
                &mut self,
                wire: FixedWire,
                row: usize,
                to: impl FnOnce() -> Result<F, Error>,
            ) -> Result<(), Error> {
                *self
                    .fixed
                    .get_mut(wire.0)
                    .and_then(|v| v.get_mut(row))
                    .ok_or(Error::BoundsFailure)? = to()?;

                Ok(())
            }

            fn create_gate(
                &mut self,
                sa: F,
                sb: F,
                sc: F,
                sd: F,
                sm: F,
                _: impl Fn() -> Result<(F, F, F, F), Error>,
            ) -> Result<(Variable, Variable, Variable, Variable), Error> {
                let tmp = Ok((
                    Variable(Wire::A, self.sa.len()),
                    Variable(Wire::B, self.sa.len()),
                    Variable(Wire::C, self.sa.len()),
                    Variable(Wire::D, self.sa.len()),
                ));
                self.sa.push(sa);
                self.sb.push(sb);
                self.sc.push(sc);
                self.sd.push(sd);
                self.sm.push(sm);
                tmp
            }
        }

        let mut meta = MetaCircuit::default();
        let config = ConcreteCircuit::configure(&mut meta);

        let mut assembly: Assembly<C::Scalar> = Assembly {
            sa: vec![],
            sb: vec![],
            sc: vec![],
            sd: vec![],
            sm: vec![],
            fixed: vec![vec![C::Scalar::zero(); params.n as usize]; meta.num_fixed_wires],
        };

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

        let fixed_commitments = assembly
            .fixed
            .iter()
            .map(|poly| params.commit_lagrange(poly, C::Scalar::one()).to_affine())
            .collect();

        let domain = EvaluationDomain::new(GATE_DEGREE, params.k);

        let sa = domain.obtain_coset(assembly.sa);
        let sb = domain.obtain_coset(assembly.sb);
        let sc = domain.obtain_coset(assembly.sc);
        let sd = domain.obtain_coset(assembly.sd);
        let sm = domain.obtain_coset(assembly.sm);

        let fixed_polys = assembly
            .fixed
            .into_iter()
            .map(|poly| domain.obtain_coset(poly))
            .collect();

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

            fixed_commitments,
            fixed_polys,
            meta,
        })
    }
}

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

        let sa_poly = domain.obtain_poly(assembly.sa);
        let sb_poly = domain.obtain_poly(assembly.sb);
        let sc_poly = domain.obtain_poly(assembly.sc);
        let sd_poly = domain.obtain_poly(assembly.sd);
        let sm_poly = domain.obtain_poly(assembly.sm);
        let sa_coset = domain.obtain_coset(sa_poly.clone(), 0);
        let sb_coset = domain.obtain_coset(sb_poly.clone(), 0);
        let sc_coset = domain.obtain_coset(sc_poly.clone(), 0);
        let sd_coset = domain.obtain_coset(sd_poly.clone(), 0);
        let sm_coset = domain.obtain_coset(sm_poly.clone(), 0);

        let fixed_polys = assembly
            .fixed
            .into_iter()
            .map(|poly| {
                let coeffs = domain.obtain_poly(poly);
                let coset = domain.obtain_coset(coeffs.clone(), 0);
                (coeffs, coset)
            })
            .collect();

        Ok(SRS {
            sa: (sa_coset, sa_poly),
            sb: (sb_coset, sb_poly),
            sc: (sc_coset, sc_poly),
            sd: (sd_coset, sd_poly),
            sm: (sm_coset, sm_poly),
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

use super::{
    circuit::{AdviceWire, Circuit, ConstraintSystem, FixedWire, MetaCircuit, Variable},
    domain::EvaluationDomain,
    Error, SRS,
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
            fixed: Vec<Vec<F>>,
            copy: Vec<Vec<Variable>>,
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

            fn assign_copy(&mut self, left: Variable, right: Variable) -> Result<(), Error> {
                *self
                    .copy
                    .get_mut((left.0).0)
                    .and_then(|v| v.get_mut(left.1))
                    .ok_or(Error::BoundsFailure)? = right;

                Ok(())
            }
        }

        let mut meta = MetaCircuit::default();
        let config = ConcreteCircuit::configure(&mut meta);

        let mut assembly: Assembly<C::Scalar> = Assembly {
            fixed: vec![vec![C::Scalar::zero(); params.n as usize]; meta.num_fixed_wires],
            copy: vec![
                vec![Variable::new(AdviceWire(0), 0); params.n as usize];
                meta.num_advice_wires
            ],
        };

        // Synthesize the circuit to obtain SRS
        circuit.synthesize(&mut assembly, config)?;

        let fixed_commitments = assembly
            .fixed
            .iter()
            .map(|poly| params.commit_lagrange(poly, C::Scalar::one()).to_affine())
            .collect();

        let mut degree = 1;
        for poly in meta.gates.iter() {
            degree = std::cmp::max(degree, poly.degree());
        }

        let domain = EvaluationDomain::new(degree as u32, params.k);

        let fixed_polys: Vec<_> = assembly
            .fixed
            .into_iter()
            .map(|poly| domain.obtain_poly(poly))
            .collect();

        let fixed_cosets = meta
            .fixed_queries
            .iter()
            .map(|&(wire, at)| {
                let poly = fixed_polys[wire.0].clone();
                domain.obtain_coset(poly, at)
            })
            .collect();

        Ok(SRS {
            domain,
            fixed_commitments,
            fixed_polys,
            fixed_cosets,
            meta,
        })
    }
}

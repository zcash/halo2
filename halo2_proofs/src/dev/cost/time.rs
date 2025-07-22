use std::{
    iter::{self, Sum},
    marker::PhantomData,
    ops::{Add, Mul},
    time::{Duration, Instant},
};

use group::{ff::Field, prime::PrimeGroup, Group};
use pasta_curves::arithmetic::{CurveAffine, CurveExt};

use super::{CircuitCost, MarginalProofSize, ProofContribution, ProofSize};
use crate::{
    plonk::Circuit,
    transcript::{Blake2bRead, Transcript, TranscriptRead},
};

impl<C: CurveExt, ConcreteCircuit: Circuit<<C as Group>::Scalar>> CircuitCost<C, ConcreteCircuit> {
    /// Returns the marginal verifying cost per instance of this circuit.
    pub fn marginal_verifying(&self) -> MarginalVerifyingCost<C::AffineExt> {
        let chunks = self.permutation_chunks();

        MarginalVerifyingCost {
            // Transcript:
            // - the marginal proof size
            proof: self.marginal_proof_size(),

            // Cells:
            // - 1 polynomial commitment per instance column per instance
            // - 1 commitment per instance column per instance
            instance_columns: self.instance_columns,

            // Gates:
            gates: TimeContribution::new(0, self.gate_expressions.iter().copied()),

            // Lookup arguments:
            // - 7 additions per lookup argument per instance
            // - 6 multiplications per lookup argument per instance
            lookups: TimeContribution::new(0, [(1, 1), (1, 2), (1, 1), (4, 2)].into_iter())
                * self.lookups,

            // Global permutation argument:
            // - 5 * chunks + 3 additions and multiplications per instance
            equality: TimeContribution::new(
                0,
                [(1, 1)]
                    .into_iter()
                    .chain((0..chunks - 1).map(|_| (1, 1)))
                    .chain((0..chunks).map(|_| (4 * chunks + 3, 4 * chunks + 3))),
            ),

            _marker: PhantomData::default(),
        }
    }

    /// Returns the verifying cost for the given number of instances of this circuit.
    pub fn verifying(&self, instances: usize) -> VerifyingCost<C::AffineExt> {
        let marginal = self.marginal_verifying();

        VerifyingCost {
            // Transcript:
            // - the proof
            // - marginal cost per instance
            proof: self.proof_size(instances),
            instance_columns: marginal.instance_columns * instances,

            // - Verifying key
            // - 5 challenges

            // Gates:
            // - marginal cost per instance
            gates: marginal.gates * instances,

            // Lookup arguments:
            // - marginal cost per instance
            lookups: marginal.lookups * instances,

            // Global permutation argument:
            // - marginal cost per instance
            // - TODO: global cost
            equality: marginal.equality * instances,

            // Vanishing argument:
            // - expressions + 1 commitments
            // - 1 random_poly eval
            vanishing: TimeContribution::new(0, [(1, 1)].into_iter()),

            // Multiopening argument:
            // - TODO: point set evals
            // - TODO: Lagrange interpolation per point set
            // - TODO: inversions
            // - 2 additions and mults per point in multiopen argument
            // - 2 additions per set of points in multiopen argument
            // - 1 multiplication per set of points in multiopen argument
            multiopen: self
                .point_sets
                .iter()
                .map(|points| TimeContribution::new(0, (2 * points + 2, 2 * points + 1).into()))
                .sum(),

            // Polycommit:
            // - s_poly commitment
            // - inner product argument (2 * k round commitments)
            // - a
            // - xi
            polycomm: ProofContribution::new(1 + 2 * self.k, 2),

            _marker: PhantomData::default(),
        }
    }
}

/// The marginal time cost of verifying a specific Halo 2 proof, broken down into its
/// contributing factors.
#[derive(Clone, Debug)]
pub struct MarginalVerifyingCost<C: CurveAffine> {
    proof: MarginalProofSize<C::Curve>,
    instance_columns: usize,
    gates: TimeContribution,
    lookups: TimeContribution,
    equality: TimeContribution,
    _marker: PhantomData<C>,
}

impl<C: CurveAffine> MarginalVerifyingCost<C> {
    fn transcript_inputs(&self) -> ProofContribution {
        self.proof.instance + self.proof.advice + self.proof.lookups + self.proof.equality
    }

    fn expressions(&self) -> ExpressionCost {
        iter::empty()
            .chain(self.gates.expressions.iter())
            .chain(self.lookups.expressions.iter())
            .chain(self.equality.expressions.iter())
            .sum()
    }

    /// Estimates the concrete time cost for verifying this proof.
    #[cfg(feature = "dev-cost")]
    pub fn estimate(&self) -> TimeCost {
        TimeCost::estimate::<C>(
            self.instance_columns,
            self.transcript_inputs(),
            self.expressions(),
        )
    }

    /// Evaluates the concrete time cost for verifying this proof.
    pub fn evaluate(&self, single_field_add: Duration, single_field_mul: Duration) -> TimeCost {
        TimeCost::evaluate::<C>(
            self.instance_columns,
            self.transcript_inputs(),
            self.expressions(),
            single_field_add,
            single_field_mul,
        )
    }
}

/// The time cost of verifying a specific Halo 2 proof, broken down into its contributing
/// factors.
#[derive(Clone, Debug)]
pub struct VerifyingCost<C: CurveAffine> {
    proof: ProofSize<C::Curve>,
    instance_columns: usize,
    gates: TimeContribution,
    lookups: TimeContribution,
    equality: TimeContribution,
    vanishing: TimeContribution,
    multiopen: TimeContribution,
    polycomm: TimeContribution,
    _marker: PhantomData<C>,
}

impl<C: CurveAffine> VerifyingCost<C> {
    fn transcript_inputs(&self) -> ProofContribution {
        // TODO
        self.proof.instance + self.proof.advice + self.proof.lookups + self.proof.equality
    }

    fn expressions(&self) -> ExpressionCost {
        iter::empty()
            .chain(self.gates.expressions.iter())
            .chain(self.lookups.expressions.iter())
            .chain(self.equality.expressions.iter())
            .chain(self.vanishing.expressions.iter())
            .chain(self.multiopen.expressions.iter())
            .chain(self.polycomm.expressions.iter())
            .sum()
    }

    /// Estimates the concrete time cost for verifying this proof.
    #[cfg(feature = "dev-cost")]
    pub fn estimate(&self) -> TimeCost {
        TimeCost::estimate::<C>(
            self.instance_columns,
            self.transcript_inputs(),
            self.expressions(),
        )
    }

    /// Evaluates the concrete time cost for verifying this proof.
    pub fn evaluate(&self, single_field_add: Duration, single_field_mul: Duration) -> TimeCost {
        TimeCost::evaluate::<C>(
            self.instance_columns,
            self.transcript_inputs(),
            self.expressions(),
            single_field_add,
            single_field_mul,
        )
    }
}

/// The estimated time cost of proving or verifying a specific Halo 2 circuit.
#[derive(Clone, Copy, Debug)]
pub struct TimeCost {
    transcript: Duration,
    expressions: Duration,
}

impl TimeCost {
    #[cfg(feature = "dev-cost")]
    fn estimate<C: CurveAffine>(
        instance_columns: usize,
        transcript_inputs: ProofContribution,
        expressions: ExpressionCost,
    ) -> Self {
        use rand_core::OsRng;

        let pairs = [0; 100].map(|_| (C::Scalar::random(OsRng), C::Scalar::random(OsRng)));

        let runner = |f: fn(C::Scalar, C::Scalar) -> C::Scalar| {
            let start = Instant::now();
            for _ in 0..100 {
                for (a, b) in pairs.into_iter() {
                    let _ = criterion::black_box(f(a, b));
                }
            }
            Instant::now().duration_since(start) / (100 * pairs.len() as u32)
        };

        let single_field_add = runner(|a, b| a + b);
        let single_field_mul = runner(|a, b| a * b);

        Self::evaluate::<C>(
            instance_columns,
            transcript_inputs,
            expressions,
            single_field_add,
            single_field_mul,
        )
    }

    fn evaluate<C: CurveAffine>(
        instance_columns: usize,
        transcript_inputs: ProofContribution,
        expressions: ExpressionCost,
        single_field_add: Duration,
        single_field_mul: Duration,
    ) -> Self {
        let dummy_point = C::generator();

        // Transcript cost
        let transcript = {
            let mut transcript = Blake2bRead::init(std::io::repeat(1));
            let start = Instant::now();
            for _ in 0..instance_columns {
                let _ = transcript.common_point(dummy_point).unwrap();
            }
            for _ in 0..transcript_inputs.commitments {
                let _ = transcript.read_point().unwrap();
            }
            for _ in 0..transcript_inputs.evaluations {
                let _ = transcript.read_scalar().unwrap();
            }
            Instant::now().duration_since(start)
        };

        // Expressions cost
        let expressions = single_field_add * expressions.add as u32
            + single_field_mul * (expressions.mul + expressions.scale) as u32;

        Self {
            transcript,
            expressions,
        }
    }

    /// Returns the total estimated time cost.
    pub fn total(&self) -> Duration {
        self.transcript + self.expressions
    }
}

#[derive(Clone, Copy, Debug)]
pub(super) struct ExpressionCost {
    add: usize,
    mul: usize,
    scale: usize,
}

/// Use when multiplications and scalings have the same cost.
impl From<(usize, usize)> for ExpressionCost {
    fn from((add, mul): (usize, usize)) -> Self {
        Self { add, mul, scale: 0 }
    }
}

impl From<(usize, usize, usize)> for ExpressionCost {
    fn from((add, mul, scale): (usize, usize, usize)) -> Self {
        Self { add, mul, scale }
    }
}

impl Add for ExpressionCost {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            add: self.add + rhs.add,
            mul: self.mul + rhs.mul,
            scale: self.scale + rhs.scale,
        }
    }
}

impl<'a> Sum<&'a ExpressionCost> for ExpressionCost {
    fn sum<I: Iterator<Item = &'a ExpressionCost>>(iter: I) -> Self {
        iter.fold((0, 0, 0).into(), |acc, expr| acc + *expr)
    }
}

#[derive(Clone, Debug)]
struct TimeContribution {
    polynomial_commitments: usize,
    expressions: Vec<ExpressionCost>,
}

impl TimeContribution {
    fn new<E>(polynomial_commitments: usize, expressions: impl Iterator<Item = E>) -> Self
    where
        ExpressionCost: From<E>,
    {
        Self {
            polynomial_commitments,
            expressions: expressions.map(ExpressionCost::from).collect(),
        }
    }
}

impl Add for TimeContribution {
    type Output = Self;

    fn add(self, mut rhs: Self) -> Self::Output {
        let mut expressions = self.expressions;
        expressions.append(&mut rhs.expressions);
        Self {
            polynomial_commitments: self.polynomial_commitments + rhs.polynomial_commitments,
            expressions,
        }
    }
}

impl Mul<usize> for TimeContribution {
    type Output = Self;

    fn mul(self, instances: usize) -> Self::Output {
        Self {
            polynomial_commitments: self.polynomial_commitments * instances,
            expressions: iter::repeat(self.expressions.into_iter())
                .take(instances)
                .flatten()
                .collect(),
        }
    }
}

impl Sum for TimeContribution {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(
            TimeContribution {
                polynomial_commitments: 0,
                expressions: vec![],
            },
            |acc, expr| acc + expr,
        )
    }
}

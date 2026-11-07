//! This module provides an implementation of a variant of (Turbo)[PLONK][plonk]
//! that is designed specifically for the polynomial commitment scheme described
//! in the [Halo][halo] paper.
//!
//! [halo]: https://eprint.iacr.org/2019/1021
//! [plonk]: https://eprint.iacr.org/2019/953

use blake2b_simd::Params as Blake2bParams;
use group::ff::{Field, FromUniformBytes, PrimeField};

use crate::arithmetic::CurveAffine;
use crate::poly::{
    Coeff, EvaluationDomain, ExtendedLagrangeCoeff, LagrangeCoeff, PinnedEvaluationDomain,
    Polynomial,
};
use crate::transcript::{ChallengeScalar, EncodedChallenge, Transcript};

mod assigned;
mod circuit;
mod error;
mod keygen;
mod lookup;
pub(crate) mod permutation;
mod vanishing;

mod prover;
mod verifier;

#[cfg(feature = "unstable-verifier-fingerprint")]
#[cfg_attr(docsrs, doc(cfg(feature = "unstable-verifier-fingerprint")))]
pub mod fingerprint;

pub use assigned::*;
pub use circuit::*;
pub use error::*;
pub use keygen::*;
pub use prover::*;
pub use verifier::*;

use std::{collections::HashMap, io, sync::Arc};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
enum VerifierExpressionOp {
    Constant(usize),
    Fixed(usize),
    Advice(usize),
    Instance(usize),
    Negated(usize),
    Sum(usize, usize),
    Product(usize, usize),
    Scaled(usize, usize),
    Selector,
}

// This plan is excluded from the pinned VK and transcript representative.
// Interning preserves every root's order and each operation's operand order;
// only identical pure field expressions share an evaluation slot.
struct VerifierExpressions<F> {
    operations: Vec<VerifierExpressionOp>,
    constants: Vec<F>,
    common: Vec<usize>,
    per_proof: Vec<usize>,
    roots: Vec<usize>,
}

impl<F> std::fmt::Debug for VerifierExpressions<F> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VerifierExpressions")
            .field("operations", &self.operations.len())
            .field("common", &self.common.len())
            .field("per_proof", &self.per_proof.len())
            .field("roots", &self.roots.len())
            .field("payload_bytes", &self.payload_bytes())
            .finish()
    }
}

impl<F> VerifierExpressions<F> {
    // Accounted vector payload, excluding object/allocator overhead and
    // transient compilation maps or per-verification field scratch space.
    fn payload_bytes(&self) -> usize {
        self.operations.capacity() * std::mem::size_of::<VerifierExpressionOp>()
            + self.constants.capacity() * std::mem::size_of::<F>()
            + (self.common.capacity() + self.per_proof.capacity() + self.roots.capacity())
                * std::mem::size_of::<usize>()
    }
}

impl<F: PrimeField> VerifierExpressions<F> {
    fn compile<'a>(expressions: impl IntoIterator<Item = &'a Expression<F>>) -> Self {
        struct Compiler<F: PrimeField> {
            plan: VerifierExpressions<F>,
            constants: HashMap<Vec<u8>, usize>,
            operations: HashMap<VerifierExpressionOp, usize>,
            common: Vec<bool>,
        }

        impl<F: PrimeField> Compiler<F> {
            fn constant(&mut self, scalar: F) -> usize {
                let repr = scalar.to_repr();
                if let Some(&index) = self.constants.get(repr.as_ref()) {
                    return index;
                }
                let index = self.plan.constants.len();
                self.plan.constants.push(scalar);
                self.constants.insert(repr.as_ref().to_vec(), index);
                index
            }

            fn expression(&mut self, expression: &Expression<F>) -> usize {
                let (operation, common) = match expression {
                    Expression::Constant(scalar) => {
                        (VerifierExpressionOp::Constant(self.constant(*scalar)), true)
                    }
                    Expression::Fixed(query) => (VerifierExpressionOp::Fixed(query.index), true),
                    Expression::Advice(query) => (VerifierExpressionOp::Advice(query.index), false),
                    Expression::Instance(query) => {
                        (VerifierExpressionOp::Instance(query.index), false)
                    }
                    Expression::Negated(a) => {
                        let a = self.expression(a);
                        (VerifierExpressionOp::Negated(a), self.common[a])
                    }
                    Expression::Sum(a, b) | Expression::Product(a, b) => {
                        let left = self.expression(a);
                        let right = self.expression(b);
                        let operation = if matches!(expression, Expression::Sum(_, _)) {
                            VerifierExpressionOp::Sum(left, right)
                        } else {
                            VerifierExpressionOp::Product(left, right)
                        };
                        (operation, self.common[left] && self.common[right])
                    }
                    Expression::Scaled(a, scalar) => {
                        let a = self.expression(a);
                        let scalar = self.constant(*scalar);
                        (VerifierExpressionOp::Scaled(a, scalar), self.common[a])
                    }
                    // Valid VK construction removes virtual selectors. Keep
                    // the old evaluation-time panic for a malformed internal
                    // expression rather than changing it into a scalar.
                    Expression::Selector(_) => (VerifierExpressionOp::Selector, true),
                };
                *self.operations.entry(operation).or_insert_with(|| {
                    let index = self.plan.operations.len();
                    self.plan.operations.push(operation);
                    self.common.push(common);
                    index
                })
            }
        }

        let mut compiler = Compiler {
            plan: Self {
                operations: Vec::new(),
                constants: Vec::new(),
                common: Vec::new(),
                per_proof: Vec::new(),
                roots: Vec::new(),
            },
            constants: HashMap::new(),
            operations: HashMap::new(),
            common: Vec::new(),
        };
        for expression in expressions {
            let root = compiler.expression(expression);
            compiler.plan.roots.push(root);
        }
        for (index, common) in compiler.common.into_iter().enumerate() {
            if common {
                compiler.plan.common.push(index);
            } else {
                compiler.plan.per_proof.push(index);
            }
        }
        compiler.plan
    }

    fn operation(
        &self,
        index: usize,
        values: &[F],
        fixed: &[F],
        advice: &[F],
        instance: &[F],
    ) -> F {
        match self.operations[index] {
            VerifierExpressionOp::Constant(index) => self.constants[index],
            VerifierExpressionOp::Fixed(index) => fixed[index],
            VerifierExpressionOp::Advice(index) => advice[index],
            VerifierExpressionOp::Instance(index) => instance[index],
            VerifierExpressionOp::Negated(a) => -values[a],
            VerifierExpressionOp::Sum(a, b) => values[a] + values[b],
            VerifierExpressionOp::Product(a, b) => values[a] * values[b],
            VerifierExpressionOp::Scaled(a, scalar) => values[a] * self.constants[scalar],
            VerifierExpressionOp::Selector => {
                panic!("virtual selectors are removed during optimization")
            }
        }
    }

    fn common_values(&self, fixed: &[F]) -> Vec<F> {
        let mut values = vec![F::ZERO; self.operations.len()];
        for &index in &self.common {
            values[index] = self.operation(index, &values, fixed, &[], &[]);
        }
        values
    }

    fn evaluate(&self, values: &mut [F], fixed: &[F], advice: &[F], instance: &[F]) -> Vec<F> {
        assert_eq!(values.len(), self.operations.len());
        for &index in &self.per_proof {
            values[index] = self.operation(index, values, fixed, advice, instance);
        }
        self.roots.iter().map(|&root| values[root]).collect()
    }
}

#[cfg(test)]
mod verifier_expression_tests;

/// This is a verifying key which allows for the verification of proofs for a
/// particular circuit.
#[derive(Clone, Debug)]
pub struct VerifyingKey<C: CurveAffine> {
    domain: EvaluationDomain<C::Scalar>,
    fixed_commitments: Vec<C>,
    permutation: permutation::VerifyingKey<C>,
    cs: ConstraintSystem<C::Scalar>,
    /// Cached maximum degree of `cs` (which doesn't change after construction).
    cs_degree: usize,
    /// The representative of this `VerifyingKey` in transcripts.
    transcript_repr: C::Scalar,
    /// Shared pure-field gate schedule; not part of [`Self::pinned`].
    verifier_expressions: Arc<VerifierExpressions<C::Scalar>>,
}

impl<C: CurveAffine> VerifyingKey<C>
where
    C::Scalar: FromUniformBytes<64>,
{
    fn from_parts(
        domain: EvaluationDomain<C::Scalar>,
        fixed_commitments: Vec<C>,
        permutation: permutation::VerifyingKey<C>,
        cs: ConstraintSystem<C::Scalar>,
    ) -> Self {
        // Compute cached values.
        let cs_degree = cs.degree();
        let verifier_expressions = Arc::new(VerifierExpressions::compile(
            cs.gates.iter().flat_map(|gate| gate.polynomials()),
        ));

        let mut vk = Self {
            domain,
            fixed_commitments,
            permutation,
            cs,
            cs_degree,
            // Temporary, this is not pinned.
            transcript_repr: C::Scalar::ZERO,
            verifier_expressions,
        };

        let mut hasher = Blake2bParams::new()
            .hash_length(64)
            .personal(b"Halo2-Verify-Key")
            .to_state();

        let s = format!("{:?}", vk.pinned());

        hasher.update(&(s.len() as u64).to_le_bytes());
        hasher.update(s.as_bytes());

        // Hash in final Blake2bState
        vk.transcript_repr = C::Scalar::from_uniform_bytes(hasher.finalize().as_array());

        vk
    }
}

impl<C: CurveAffine> VerifyingKey<C> {
    /// Hashes a verification key into a transcript.
    pub fn hash_into<E: EncodedChallenge<C>, T: Transcript<C, E>>(
        &self,
        transcript: &mut T,
    ) -> io::Result<()> {
        transcript.common_scalar(self.transcript_repr)?;

        Ok(())
    }

    /// Obtains a pinned representation of this verification key that contains
    /// the minimal information necessary to reconstruct the verification key.
    pub fn pinned(&self) -> PinnedVerificationKey<'_, C> {
        PinnedVerificationKey {
            base_modulus: C::Base::MODULUS,
            scalar_modulus: C::Scalar::MODULUS,
            domain: self.domain.pinned(),
            fixed_commitments: &self.fixed_commitments,
            permutation: &self.permutation,
            cs: self.cs.pinned(),
        }
    }
}

/// Minimal representation of a verification key that can be used to identify
/// its active contents.
#[allow(dead_code)]
#[derive(Debug)]
pub struct PinnedVerificationKey<'a, C: CurveAffine> {
    base_modulus: &'static str,
    scalar_modulus: &'static str,
    domain: PinnedEvaluationDomain<'a, C::Scalar>,
    cs: PinnedConstraintSystem<'a, C::Scalar>,
    fixed_commitments: &'a Vec<C>,
    permutation: &'a permutation::VerifyingKey<C>,
}
/// This is a proving key which allows for the creation of proofs for a
/// particular circuit.
#[derive(Clone, Debug)]
pub struct ProvingKey<C: CurveAffine> {
    vk: VerifyingKey<C>,
    l0: Polynomial<C::Scalar, ExtendedLagrangeCoeff>,
    l_blind: Polynomial<C::Scalar, ExtendedLagrangeCoeff>,
    l_last: Polynomial<C::Scalar, ExtendedLagrangeCoeff>,
    fixed_values: Vec<Polynomial<C::Scalar, LagrangeCoeff>>,
    fixed_polys: Vec<Polynomial<C::Scalar, Coeff>>,
    fixed_cosets: Vec<Polynomial<C::Scalar, ExtendedLagrangeCoeff>>,
    permutation: permutation::ProvingKey<C>,
}

impl<C: CurveAffine> ProvingKey<C> {
    /// Get the underlying [`VerifyingKey`].
    pub fn get_vk(&self) -> &VerifyingKey<C> {
        &self.vk
    }
}

impl<C: CurveAffine> VerifyingKey<C> {
    /// Get the underlying [`EvaluationDomain`].
    pub fn get_domain(&self) -> &EvaluationDomain<C::Scalar> {
        &self.domain
    }
}

#[derive(Clone, Copy, Debug)]
struct Theta;
type ChallengeTheta<F> = ChallengeScalar<F, Theta>;

#[derive(Clone, Copy, Debug)]
struct Beta;
type ChallengeBeta<F> = ChallengeScalar<F, Beta>;

#[derive(Clone, Copy, Debug)]
struct Gamma;
type ChallengeGamma<F> = ChallengeScalar<F, Gamma>;

#[derive(Clone, Copy, Debug)]
struct Y;
type ChallengeY<F> = ChallengeScalar<F, Y>;

#[derive(Clone, Copy, Debug)]
struct X;
type ChallengeX<F> = ChallengeScalar<F, X>;

pub mod prover;
pub mod verifier;

use std::marker::PhantomData;

use halo2_proofs::{arithmetic::CurveAffine, circuit::Value};
pub use verifier::Verifier;

type InstanceColumn = Vec<Value<bool>>;
type Instance = Vec<InstanceColumn>;

trait Accumulator<C: CurveAffine> {
    /// An evaluation claim (C, z) corresponding to an old instance
    type Input;
    /// An evaluation claim (C', z') corresponding to the new accumulator
    type Output: Into<Instance> + Copy;
    // /// Witness value
    // /// (e.g. in IPA, we can put challenges into witness)
    // type Witness;

    /// Read a previous instance (C, z), encoded as a public input to
    /// the proof being checked
    fn read_instance(instances: &[Instance]) -> Self::Input;

    /// Read the new accumulator (C', z'), encoded as a public input to the
    /// proof being checked, where C' := ∑ [ß^i] G_i is a random linear
    /// combination of the commitments to IPA challenges G_i's in the old instances.
    /// The new C' is evaluated at a random challenge z'.
    fn read_new_acc(instances: &[Instance]) -> Self::Output;

    /// Check that the new accumulator (C', z') was correctly constructed from
    /// the old instances.
    fn check_new_acc(
        accs: &[Self::Input],
        new_acc: Self::Output,
        is_base_case: Value<bool>,
    ) -> Value<bool>;
}

#[derive(Clone, Copy)]
struct SplitAccumulator<C: CurveAffine>(PhantomData<C>);

impl<C: CurveAffine> Into<Instance> for SplitAccumulator<C> {
    fn into(self) -> Instance {
        vec![]
    }
}

impl<C: CurveAffine> Accumulator<C> for SplitAccumulator<C> {
    type Input = Self;
    type Output = Self;

    fn read_instance(_instances: &[Instance]) -> Self::Input {
        Self(PhantomData)
    }

    fn read_new_acc(_instances: &[Instance]) -> Self::Input {
        Self(PhantomData)
    }

    fn check_new_acc(
        _accs: &[Self::Input],
        _new_acc: Self::Output,
        is_base_case: Value<bool>,
    ) -> Value<bool> {
        Value::known(true)
    }
}

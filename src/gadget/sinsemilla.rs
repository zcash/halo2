//! Gadget and chips for the [Sinsemilla] hash function.
//!
//! [Sinsemilla]: https://hackmd.io/iOw7-HpFQY6dPF1aFY8pAw

use std::fmt;

use crate::{
    arithmetic::FieldExt,
    circuit::{Chip, Layouter},
    plonk::Error,
};

mod chip;
pub use chip::{SinsemillaChip, SinsemillaColumns, SinsemillaConfig};

/// Domain prefix used in SWU hash-to-curve to generate S_i's.
pub const S_DOMAIN_PREFIX: &str = "z.cash:SinsemillaS";

/// Domain prefix used in SWU hash-to-curve to generate Q.
pub const Q_DOMAIN_PREFIX: &str = "z.cash:SinsemillaQ";

/// Personalization input used to generate Q
/// TODO: Decide on personalization
pub const Q_PERSONALIZATION: [u8; 4] = [0u8; 4];

/// The set of circuit instructions required to use the [`Sinsemilla`](https://zcash.github.io/halo2/design/gadgets/sinsemilla.html) gadget.
pub trait SinsemillaInstructions<F: FieldExt>: Chip<Field = F> {
    /// A message of at most `kn` bits.
    type Message: Clone + fmt::Debug;
    /// The output of `Hash`.
    type HashOutput: fmt::Debug;

    /// Return Q
    fn q() -> (F, F);

    /// Hashes the given message.
    ///
    /// TODO: Since the output is always curve point, maybe this should return
    /// `<Self as EccInstructions>::Point` instead of an associated type.
    fn hash(
        layouter: &mut impl Layouter<Self>,
        message: Self::Message,
    ) -> Result<Self::HashOutput, Error>;
}

#[test]
fn test_sinsemilla() {
    use crate::arithmetic::CurveAffine;
    use crate::circuit::layouter::SingleChip;
    use crate::pasta::{EpAffine, EqAffine};
    use crate::plonk::*;
    use crate::poly::commitment::Params;
    use crate::transcript::{Blake2bRead, Blake2bWrite};

    use std::marker::PhantomData;

    /// This represents an advice column at a certain row in the ConstraintSystem
    #[derive(Copy, Clone, Debug)]
    pub struct Variable(Column<Advice>, usize);

    struct MyCircuit<C: CurveAffine> {
        message: Vec<bool>,
        _marker_c: PhantomData<C>,
    }

    impl<'a, C: Chip, CS: Assignment<C::Field>> SingleChip<'a, C, CS> {
        fn load_new(cs: &'a mut CS, config: C::Config) -> Result<Self, Error> {
            let mut res = SingleChip::new(cs, config);

            C::load(&mut res)?;

            Ok(res)
        }
    }

    impl<C: CurveAffine> Circuit<C::Base> for MyCircuit<C> {
        type Config = SinsemillaConfig;

        fn configure(meta: &mut ConstraintSystem<C::Base>) -> Self::Config {
            let columns = SinsemillaColumns::new(
                meta.fixed_column(),
                meta.advice_column(),
                meta.advice_column(),
                meta.advice_column(),
                meta.advice_column(),
                meta.advice_column(),
            );

            SinsemillaChip::<C>::configure(meta, 11, 2, columns)
        }

        fn synthesize(
            &self,
            cs: &mut impl Assignment<C::Base>,
            config: Self::Config,
        ) -> Result<(), Error> {
            let mut layouter = SingleChip::load_new(cs, config)?;

            SinsemillaChip::<C>::hash(&mut layouter, self.message.clone())?;

            Ok(())
        }
    }

    // Initialize the polynomial commitment parameters
    let k = 11;
    let params: Params<EqAffine> = Params::new(k);
    let empty_circuit: MyCircuit<EpAffine> = MyCircuit {
        message: Vec::new(),
        _marker_c: PhantomData,
    };

    // Initialize the proving key
    let vk = keygen_vk(&params, &empty_circuit).expect("keygen_vk should not fail");
    let pk = keygen_pk(&params, vk, &empty_circuit).expect("keygen_pk should not fail");

    let circuit: MyCircuit<EpAffine> = MyCircuit {
        // 101101101101
        message: vec![
            true, false, true, true, false, true, true, false, true, true, false, false,
        ],
        _marker_c: PhantomData,
    };

    // Create a proof
    let mut transcript = Blake2bWrite::init(vec![]);
    create_proof(&params, &pk, &[circuit], &[&[]], &mut transcript)
        .expect("proof generation should not fail");
    let proof: Vec<u8> = transcript.finalize();

    let msm = params.empty_msm();
    let mut transcript = Blake2bRead::init(&proof[..]);
    let guard = verify_proof(&params, pk.get_vk(), msm, &[&[]], &mut transcript).unwrap();
    let msm = guard.clone().use_challenges();
    assert!(msm.eval());
}

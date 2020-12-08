use std::marker::PhantomData;

use super::{MinimalCs, Variable};
use crate::{
    arithmetic::FieldExt,
    plonk::{Advice, Assignment, Column, ConstraintSystem, Error, Fixed},
};

fn require<T>(val: Option<T>) -> impl FnOnce() -> Result<T, Error> {
    || val.ok_or(Error::SynthesisError)
}

struct NorGate {
    a: Column<Advice>,
    b: Column<Advice>,
    c: Column<Advice>,
    sn: Column<Fixed>,
}

impl NorGate {
    /// Implements the constraints necessary for NOR instructions.
    pub fn new<F: FieldExt>(
        meta: &mut ConstraintSystem<F>,
        a: Column<Advice>,
        b: Column<Advice>,
        c: Column<Advice>,
    ) -> Self {
        // TODO: Add notion of "selectors" to ConstraintSystem, to let it optimise for
        // fewer overall fixed columns.
        let sn = meta.fixed_column();

        meta.create_gate(|meta| {
            let a = meta.query_advice(a, 0);
            let b = meta.query_advice(b, 0);
            let c = meta.query_advice(c, 0);

            let sn = meta.query_fixed(sn, 0);

            // a NOR b = c
            //
            // Given three advice columns with cells that are already boolean-constrained:
            // Ad_1, Ad_2, Ad_3, Sel_NOR
            //
            // Sel_NOR * ((1 - a) * (1 - b) - c) = 0
            //
            // Rewrite to eliminate constant: (TODO: Or have CS provide constant?)
            // Sel_NOR * (1 - a - b + ab - c) = 0
            // Sel_NOR - Sel_NOR * (ab - a - b - c) = 0
            sn - sn * (a * b - a - b - c)
        });

        NorGate { a, b, c, sn }
    }
}

trait BoolAssign {
    type Bool;

    fn public_boolean<V>(&mut self, value: V) -> Result<Self::Bool, Error>
    where
        V: FnOnce() -> Result<bool, Error>;
    fn input_boolean<V>(&mut self, value: V) -> Result<Self::Bool, Error>
    where
        V: FnOnce() -> Result<bool, Error>;
}

trait Nor {
    type Bool;

    fn nor<A, B>(&mut self, a: A, b: B) -> Result<(Self::Bool, Self::Bool, Self::Bool), Error>
    where
        A: FnOnce() -> Result<bool, Error>,
        B: FnOnce() -> Result<bool, Error>;
}

struct Boolean {
    var: Variable,
    val: Option<bool>,
}

impl Boolean {
    fn new(var: Variable, val: Option<bool>) -> Self {
        Boolean { var, val }
    }

    fn nor<F: FieldExt, CS: MinimalCs<F> + Nor<Bool = Self>>(
        self,
        cs: &mut CS,
        other: Self,
    ) -> Result<Self, Error> {
        let (a, b, c) = cs.nor(require(self.val), require(other.val))?;
        cs.constrain_equal(self.var, a.var)?;
        cs.constrain_equal(other.var, b.var)?;
        Ok(c)
    }
}

struct NorChip<'a, F: FieldExt, CS: Assignment<F> + 'a> {
    cs: &'a mut CS,
    nor_gate: NorGate,
    _marker: PhantomData<F>,
}

impl<'a, F: FieldExt, CS: Assignment<F> + 'a> BoolAssign for NorChip<'a, F, CS> {
    type Bool = Boolean;

    fn public_boolean<V>(&mut self, value: V) -> Result<Self::Bool, Error>
    where
        V: FnOnce() -> Result<bool, Error>,
    {
        todo!()
    }

    fn input_boolean<V>(&mut self, value: V) -> Result<Self::Bool, Error>
    where
        V: FnOnce() -> Result<bool, Error>,
    {
        todo!()
    }
}

impl<'a, F: FieldExt, CS: Assignment<F> + 'a> Nor for NorChip<'a, F, CS> {
    type Bool = Boolean;

    fn nor<A, B>(&mut self, a: A, b: B) -> Result<(Self::Bool, Self::Bool, Self::Bool), Error>
    where
        A: FnOnce() -> Result<bool, Error>,
        B: FnOnce() -> Result<bool, Error>,
    {
        // This should probably be defined by the caller
        // (would need to give the caller knowledge of what )
        let index = self.current_gate;
        self.current_gate += 1;

        let mut a_val = None;
        let mut b_val = None;
        let mut c_val = None;

        fn field_bool<F: FieldExt>(val: Option<bool>) -> Result<F, Error> {
            val.map(|b| if b { F::one() } else { F::zero() })
                .ok_or(Error::SynthesisError)
        }

        self.cs.assign_advice(self.nor_gate.a, index, || {
            a_val = Some(a()?);
            field_bool(a_val)
        })?;
        self.cs.assign_advice(self.nor_gate.b, index, || {
            b_val = Some(b()?);
            field_bool(b_val)
        })?;
        self.cs.assign_advice(self.nor_gate.c, index, || {
            c_val = a_val.and_then(|a| b_val.map(|b| !(a | b)));
            field_bool(c_val)
        })?;

        self.cs
            .assign_fixed(self.nor_gate.sn, index, || Ok(F::one()))?;
        Ok((
            Boolean::new(Variable(self.nor_gate.a, index), a_val),
            Boolean::new(Variable(self.nor_gate.b, index), b_val),
            Boolean::new(Variable(self.nor_gate.c, index), c_val),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::{require, BoolAssign, Nor, NorChip, NorGate};
    use crate::{
        arithmetic::{Curve, FieldExt},
        plonk::{keygen, Assignment, Circuit, ConstraintSystem, Error, Proof},
        poly::commitment::{Blind, Params},
        transcript::DummyHash,
        tweedle::{EqAffine, Fp, Fq},
    };

    const K: u32 = 5;

    #[test]
    fn bool_test() {
        // Initialize the polynomial commitment parameters
        let params: Params<EqAffine> = Params::new::<DummyHash<Fq>>(K);

        struct MyConfig {
            nor_gate: NorGate,
        }

        struct MyChip<'a, F: FieldExt, CS: Assignment<F> + 'a> {
            cs: &'a mut CS,
            nor_chip: NorChip<'a, F, CS>,
        }

        impl<'a, F: FieldExt, CS: Assignment<F> + 'a> MyChip<'a, F, CS> {
            fn new(cs: &'a mut CS, config: MyConfig) -> Self {
                MyChip {
                    cs,
                    nor_chip: NorChip {
                        cs,
                        nor_gate: config.nor_gate,
                        _marker: Default::default(),
                    },
                }
            }
        }

        impl<'a, F: FieldExt, CS: Assignment<F> + 'a> BoolAssign for MyChip<'a, F, CS> {
            type Bool = <NorChip<'a, F, CS> as Nor>::Bool;

            fn public_boolean<V>(&mut self, value: V) -> Result<Self::Bool, Error>
            where
                V: FnOnce() -> Result<bool, Error>,
            {
                self.nor_chip.public_boolean(value)
            }

            fn input_boolean<V>(&mut self, value: V) -> Result<Self::Bool, Error>
            where
                V: FnOnce() -> Result<bool, Error>,
            {
                self.nor_chip.input_boolean(value)
            }
        }

        impl<'a, F: FieldExt, CS: Assignment<F> + 'a> Nor for MyChip<'a, F, CS> {
            type Bool = <NorChip<'a, F, CS> as Nor>::Bool;

            fn nor<A, B>(
                &mut self,
                a: A,
                b: B,
            ) -> Result<(Self::Bool, Self::Bool, Self::Bool), Error>
            where
                A: FnOnce() -> Result<bool, Error>,
                B: FnOnce() -> Result<bool, Error>,
            {
                self.nor_chip.nor(a, b)
            }
        }

        struct MyCircuit {
            a: Option<bool>,
            b: Option<bool>,
            c: Option<bool>,
        }

        impl<F: FieldExt> Circuit<F> for MyCircuit {
            type Config = MyConfig;

            fn configure(meta: &mut ConstraintSystem<F>) -> MyConfig {
                // Assign all the advice columns we will need.
                let a = meta.advice_column();
                let b = meta.advice_column();
                let c = meta.advice_column();

                // Configure our gadgets.
                let nor_gate = NorGate::new(meta, a, b, c);

                MyConfig { nor_gate }
            }

            fn synthesize(
                &self,
                cs: &mut impl Assignment<F>,
                config: MyConfig,
            ) -> Result<(), Error> {
                let mut cs = MyChip::new(cs, config);

                let mut a = cs.public_boolean(require(self.a))?;
                let mut b = cs.input_boolean(require(self.b))?;
                let c = a.nor(&mut cs, b)?;

                Ok(())
            }
        }

        let a = false;
        let b = true;
        let c = !(a | b);

        let empty_circuit: MyCircuit = MyCircuit {
            a: None,
            b: None,
            c: None,
        };

        let circuit: MyCircuit = MyCircuit {
            a: Some(a),
            b: Some(b),
            c: Some(c),
        };

        // Initialize the proving key
        let pk = keygen(&params, &empty_circuit).expect("keygen should not fail");

        let mut pubinputs = pk.get_vk().get_domain().empty_lagrange();
        // pubinputs[0] = aux; // TODO
        let pubinput = params
            .commit_lagrange(&pubinputs, Blind::default())
            .to_affine();

        for _ in 0..100 {
            // Create a proof
            let proof = Proof::create::<DummyHash<Fq>, DummyHash<Fp>, _>(
                &params,
                &pk,
                &circuit,
                &[pubinputs.clone()],
            )
            .expect("proof generation should not fail");

            let pubinput_slice = &[pubinput];
            let msm = params.empty_msm();
            let guard = proof
                .verify::<DummyHash<Fq>, DummyHash<Fp>>(&params, pk.get_vk(), msm, pubinput_slice)
                .unwrap();
            {
                let msm = guard.clone().use_challenges();
                assert!(msm.eval());
            }
            {
                let g = guard.compute_g();
                let (msm, _) = guard.clone().use_g(g);
                assert!(msm.eval());
            }
            let msm = guard.clone().use_challenges();
            assert!(msm.clone().eval());
            let guard = proof
                .verify::<DummyHash<Fq>, DummyHash<Fp>>(&params, pk.get_vk(), msm, pubinput_slice)
                .unwrap();
            {
                let msm = guard.clone().use_challenges();
                assert!(msm.eval());
            }
            {
                let g = guard.compute_g();
                let (msm, _) = guard.clone().use_g(g);
                assert!(msm.eval());
            }
        }
    }
}

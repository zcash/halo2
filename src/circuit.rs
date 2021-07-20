//! The Orchard Action circuit implementation.

use std::mem;

use group::{Curve, GroupEncoding};
use halo2::{
    circuit::{Layouter, SimpleFloorPlanner},
    plonk::{self, Advice, Column, Expression, Fixed, Instance as InstanceColumn, Selector},
    poly::Rotation,
    transcript::{Blake2bRead, Blake2bWrite},
};
use pasta_curves::{
    arithmetic::{CurveAffine, FieldExt},
    pallas, vesta,
};

use crate::{
    constants::{
        load::{NullifierK, OrchardFixedBasesFull, ValueCommitV},
        MERKLE_DEPTH_ORCHARD,
    },
    keys::{
        CommitIvkRandomness, DiversifiedTransmissionKey, NullifierDerivingKey, SpendValidatingKey,
    },
    note::{
        commitment::{NoteCommitTrapdoor, NoteCommitment},
        nullifier::Nullifier,
        ExtractedNoteCommitment,
    },
    primitives::{
        poseidon::{self, ConstantLength},
        redpallas::{SpendAuth, VerificationKey},
    },
    spec::NonIdentityPallasPoint,
    tree::Anchor,
    value::{NoteValue, ValueCommitTrapdoor, ValueCommitment},
};
use gadget::{
    ecc::{
        chip::{EccChip, EccConfig},
        FixedPoint, FixedPointBaseField, FixedPointShort, Point,
    },
    poseidon::{
        Hash as PoseidonHash, Pow5T3Chip as PoseidonChip, Pow5T3Config as PoseidonConfig,
        StateWord, Word,
    },
    sinsemilla::{
        chip::{SinsemillaChip, SinsemillaConfig, SinsemillaHashDomains},
        commit_ivk::CommitIvkConfig,
        merkle::{
            chip::{MerkleChip, MerkleConfig},
            MerklePath,
        },
        note_commit::NoteCommitConfig,
    },
    utilities::{
        copy,
        plonk::{PLONKChip, PLONKConfig, PLONKInstructions},
        CellValue, UtilitiesInstructions, Var,
    },
};

use std::convert::TryInto;

pub(crate) mod gadget;

/// Size of the Orchard circuit.
// FIXME: This circuit should fit within 2^11 rows.
const K: u32 = 12;

// Absolute offsets for public inputs.
const ANCHOR: usize = 0;
const NF_OLD: usize = 1;
const CV_NET_X: usize = 2;
const CV_NET_Y: usize = 3;
const RK_X: usize = 4;
const RK_Y: usize = 5;
const CMX: usize = 6;
const ENABLE_SPEND: usize = 7;
const ENABLE_OUTPUT: usize = 8;

/// Configuration needed to use the Orchard Action circuit.
#[derive(Clone, Debug)]
pub struct Config {
    primary: Column<InstanceColumn>,
    constants: Column<Fixed>,
    q_orchard: Selector,
    advices: [Column<Advice>; 10],
    ecc_config: EccConfig,
    poseidon_config: PoseidonConfig<pallas::Base>,
    plonk_config: PLONKConfig,
    merkle_config_1: MerkleConfig,
    merkle_config_2: MerkleConfig,
    sinsemilla_config_1: SinsemillaConfig,
    sinsemilla_config_2: SinsemillaConfig,
    commit_ivk_config: CommitIvkConfig,
    old_note_commit_config: NoteCommitConfig,
    new_note_commit_config: NoteCommitConfig,
}

/// The Orchard Action circuit.
#[derive(Debug, Default)]
pub struct Circuit {
    pub(crate) path: Option<[pallas::Base; MERKLE_DEPTH_ORCHARD]>,
    pub(crate) pos: Option<u32>,
    pub(crate) g_d_old: Option<NonIdentityPallasPoint>,
    pub(crate) pk_d_old: Option<DiversifiedTransmissionKey>,
    pub(crate) v_old: Option<NoteValue>,
    pub(crate) rho_old: Option<Nullifier>,
    pub(crate) psi_old: Option<pallas::Base>,
    pub(crate) rcm_old: Option<NoteCommitTrapdoor>,
    pub(crate) cm_old: Option<NoteCommitment>,
    pub(crate) alpha: Option<pallas::Scalar>,
    pub(crate) ak: Option<SpendValidatingKey>,
    pub(crate) nk: Option<NullifierDerivingKey>,
    pub(crate) rivk: Option<CommitIvkRandomness>,
    pub(crate) g_d_new_star: Option<[u8; 32]>,
    pub(crate) pk_d_new_star: Option<[u8; 32]>,
    pub(crate) v_new: Option<NoteValue>,
    pub(crate) psi_new: Option<pallas::Base>,
    pub(crate) rcm_new: Option<NoteCommitTrapdoor>,
    pub(crate) rcv: Option<ValueCommitTrapdoor>,
}

impl UtilitiesInstructions<pallas::Base> for Circuit {
    type Var = CellValue<pallas::Base>;
}

impl plonk::Circuit<pallas::Base> for Circuit {
    type Config = Config;
    type FloorPlanner = SimpleFloorPlanner;

    fn without_witnesses(&self) -> Self {
        Self::default()
    }

    fn configure(meta: &mut plonk::ConstraintSystem<pallas::Base>) -> Self::Config {
        // Advice columns used in the Orchard circuit.
        let advices = [
            meta.advice_column(),
            meta.advice_column(),
            meta.advice_column(),
            meta.advice_column(),
            meta.advice_column(),
            meta.advice_column(),
            meta.advice_column(),
            meta.advice_column(),
            meta.advice_column(),
            meta.advice_column(),
        ];

        // Constrain v_old - v_new = magnitude * sign
        // Either v_old = 0, or anchor equals public input
        // Constrain v_old = 0 or enable_spends = 1.
        // Constrain v_new = 0 or enable_outputs = 1.
        let q_orchard = meta.selector();
        meta.create_gate("Orchard circuit checks", |meta| {
            let q_orchard = meta.query_selector(q_orchard);
            let v_old = meta.query_advice(advices[0], Rotation::cur());
            let v_new = meta.query_advice(advices[1], Rotation::cur());
            let magnitude = meta.query_advice(advices[2], Rotation::cur());
            let sign = meta.query_advice(advices[3], Rotation::cur());

            let anchor = meta.query_advice(advices[4], Rotation::cur());
            let pub_input_anchor = meta.query_advice(advices[5], Rotation::cur());

            let one = Expression::Constant(pallas::Base::one());
            let not_enable_spends = one.clone() - meta.query_advice(advices[6], Rotation::cur());
            let not_enable_outputs = one - meta.query_advice(advices[7], Rotation::cur());

            std::array::IntoIter::new([
                (
                    "v_old - v_new = magnitude * sign",
                    v_old.clone() - v_new.clone() - magnitude * sign,
                ),
                (
                    "Either v_old = 0, or anchor equals public input",
                    v_old.clone() * (anchor - pub_input_anchor),
                ),
                ("v_old = 0 or enable_spends = 1", v_old * not_enable_spends),
                (
                    "v_new = 0 or enable_outputs = 1",
                    v_new * not_enable_outputs,
                ),
            ])
            .map(move |(name, poly)| (name, q_orchard.clone() * poly))
        });

        // Fixed columns for the Sinsemilla generator lookup table
        let table_idx = meta.fixed_column();
        let lookup = (table_idx, meta.fixed_column(), meta.fixed_column());

        // Instance column used for public inputs
        let primary = meta.instance_column();
        meta.enable_equality(primary.into());

        // Shared fixed column used to load constants
        let constants = meta.fixed_column();
        meta.enable_constant(constants);

        // Permutation over all advice columns.
        for advice in advices.iter() {
            meta.enable_equality((*advice).into());
        }

        // Configuration for curve point operations.
        // This uses 10 advice columns and spans the whole circuit.
        let ecc_config = EccChip::configure(meta, advices, table_idx);

        // Configuration for the Poseidon hash.
        let poseidon_config = PoseidonChip::configure(
            meta,
            poseidon::OrchardNullifier,
            [advices[0], advices[1], advices[2]],
            advices[3],
        );

        // Configuration for standard PLONK (addition and multiplication).
        let plonk_config = PLONKChip::configure(meta, [advices[0], advices[1], advices[2]]);

        // Configuration for a Sinsemilla hash instantiation and a
        // Merkle hash instantiation using this Sinsemilla instance.
        // Since the Sinsemilla config uses only 5 advice columns,
        // we can fit two instances side-by-side.
        let (sinsemilla_config_1, merkle_config_1) = {
            let sinsemilla_config_1 =
                SinsemillaChip::configure(meta, advices[..5].try_into().unwrap(), lookup);
            let merkle_config_1 = MerkleChip::configure(meta, sinsemilla_config_1.clone());

            (sinsemilla_config_1, merkle_config_1)
        };

        // Configuration for a Sinsemilla hash instantiation and a
        // Merkle hash instantiation using this Sinsemilla instance.
        // Since the Sinsemilla config uses only 5 advice columns,
        // we can fit two instances side-by-side.
        let (sinsemilla_config_2, merkle_config_2) = {
            let sinsemilla_config_2 =
                SinsemillaChip::configure(meta, advices[5..].try_into().unwrap(), lookup);
            let merkle_config_2 = MerkleChip::configure(meta, sinsemilla_config_2.clone());

            (sinsemilla_config_2, merkle_config_2)
        };

        // Configuration to handle decomposition and canonicity checking
        // for CommitIvk.
        let commit_ivk_config =
            CommitIvkConfig::configure(meta, advices, sinsemilla_config_1.clone());

        // Configuration to handle decomposition and canonicity checking
        // for NoteCommit_old.
        let old_note_commit_config =
            NoteCommitConfig::configure(meta, advices, sinsemilla_config_1.clone());

        // Configuration to handle decomposition and canonicity checking
        // for NoteCommit_new.
        let new_note_commit_config =
            NoteCommitConfig::configure(meta, advices, sinsemilla_config_2.clone());

        Config {
            primary,
            constants,
            q_orchard,
            advices,
            ecc_config,
            poseidon_config,
            plonk_config,
            merkle_config_1,
            merkle_config_2,
            sinsemilla_config_1,
            sinsemilla_config_2,
            commit_ivk_config,
            old_note_commit_config,
            new_note_commit_config,
        }
    }

    fn synthesize(
        &self,
        config: Self::Config,
        mut layouter: impl Layouter<pallas::Base>,
    ) -> Result<(), plonk::Error> {
        // Load the Sinsemilla generator lookup table used by the whole circuit.
        SinsemillaChip::load(config.sinsemilla_config_1.clone(), &mut layouter)?;

        // Construct the ECC chip.
        let ecc_chip = config.ecc_chip();

        // Witness private inputs that are used across multiple checks.
        let (rho_old, psi_old, cm_old, g_d_old, ak, nk, v_old, v_new) = {
            // Witness psi_old
            let psi_old = self.load_private(
                layouter.namespace(|| "witness psi_old"),
                config.advices[0],
                self.psi_old,
            )?;

            // Witness rho_old
            let rho_old = self.load_private(
                layouter.namespace(|| "witness rho_old"),
                config.advices[0],
                self.rho_old.map(|rho| rho.0),
            )?;

            // Witness cm_old
            let cm_old = Point::new(
                config.ecc_chip(),
                layouter.namespace(|| "cm_old"),
                self.cm_old.as_ref().map(|cm| cm.to_affine()),
            )?;

            // Witness g_d_old
            let g_d_old = Point::new(
                config.ecc_chip(),
                layouter.namespace(|| "gd_old"),
                self.g_d_old.as_ref().map(|gd| gd.to_affine()),
            )?;

            // Witness ak.
            let ak: Option<pallas::Point> = self.ak.as_ref().map(|ak| ak.into());
            let ak = Point::new(
                ecc_chip.clone(),
                layouter.namespace(|| "ak"),
                ak.map(|ak| ak.to_affine()),
            )?;

            // Witness nk.
            let nk = self.load_private(
                layouter.namespace(|| "witness nk"),
                config.advices[0],
                self.nk.map(|nk| *nk),
            )?;

            // Witness v_old.
            let v_old = self.load_private(
                layouter.namespace(|| "witness v_old"),
                config.advices[0],
                self.v_old
                    .map(|v_old| pallas::Base::from_u64(v_old.inner())),
            )?;

            // Witness v_new.
            let v_new = self.load_private(
                layouter.namespace(|| "witness v_new"),
                config.advices[0],
                self.v_new
                    .map(|v_new| pallas::Base::from_u64(v_new.inner())),
            )?;

            (rho_old, psi_old, cm_old, g_d_old, ak, nk, v_old, v_new)
        };

        // Merkle path validity check.
        let anchor = {
            let merkle_inputs = MerklePath {
                chip_1: config.merkle_chip_1(),
                chip_2: config.merkle_chip_2(),
                domain: SinsemillaHashDomains::MerkleCrh,
                leaf_pos: self.pos,
                path: self.path,
            };
            let leaf = *cm_old.extract_p().inner();
            merkle_inputs.calculate_root(layouter.namespace(|| "MerkleCRH"), leaf)?
        };

        // Value commitment integrity.
        let v_net = {
            // v_net = v_old - v_new
            let v_net = {
                // v_old, v_new are guaranteed to be 64-bit values. Therefore, we can
                // move them into the base field.
                let v_old = self
                    .v_old
                    .map(|v_old| pallas::Base::from_u64(v_old.inner()));
                let v_new = self
                    .v_new
                    .map(|v_new| pallas::Base::from_u64(v_new.inner()));

                let magnitude_sign = v_old.zip(v_new).map(|(v_old, v_new)| {
                    let is_negative = v_old < v_new;
                    let magnitude = if is_negative {
                        v_new - v_old
                    } else {
                        v_old - v_new
                    };
                    let sign = if is_negative {
                        -pallas::Base::one()
                    } else {
                        pallas::Base::one()
                    };
                    (magnitude, sign)
                });

                let magnitude = self.load_private(
                    layouter.namespace(|| "v_net magnitude"),
                    config.advices[9],
                    magnitude_sign.map(|m_s| m_s.0),
                )?;
                let sign = self.load_private(
                    layouter.namespace(|| "v_net sign"),
                    config.advices[9],
                    magnitude_sign.map(|m_s| m_s.1),
                )?;
                (magnitude, sign)
            };

            // commitment = [v_net] ValueCommitV
            let (commitment, _) = {
                let value_commit_v = ValueCommitV::get();
                let value_commit_v = FixedPointShort::from_inner(ecc_chip.clone(), value_commit_v);
                value_commit_v.mul(layouter.namespace(|| "[v_net] ValueCommitV"), v_net)?
            };

            // blind = [rcv] ValueCommitR
            let (blind, _rcv) = {
                let rcv = self.rcv.as_ref().map(|rcv| **rcv);
                let value_commit_r = OrchardFixedBasesFull::ValueCommitR;
                let value_commit_r = FixedPoint::from_inner(ecc_chip.clone(), value_commit_r);

                // [rcv] ValueCommitR
                value_commit_r.mul(layouter.namespace(|| "[rcv] ValueCommitR"), rcv)?
            };

            // [v_net] ValueCommitV + [rcv] ValueCommitR
            let cv_net = commitment.add(layouter.namespace(|| "cv_net"), &blind)?;

            // Constrain cv_net to equal public input
            layouter.constrain_instance(cv_net.inner().x().cell(), config.primary, CV_NET_X)?;
            layouter.constrain_instance(cv_net.inner().y().cell(), config.primary, CV_NET_Y)?;

            v_net
        };

        // Nullifier integrity
        let nf_old = {
            // nk_rho_old = poseidon_hash(nk, rho_old)
            let nk_rho_old = {
                let message = [nk, rho_old];

                let poseidon_message = layouter.assign_region(
                    || "load message",
                    |mut region| {
                        let mut message_word = |i: usize| {
                            let value = message[i].value();
                            let var = region.assign_advice(
                                || format!("load message_{}", i),
                                config.poseidon_config.state[i],
                                0,
                                || value.ok_or(plonk::Error::SynthesisError),
                            )?;
                            region.constrain_equal(var, message[i].cell())?;
                            Ok(Word::<_, _, poseidon::OrchardNullifier, 3, 2> {
                                inner: StateWord::new(var, value),
                            })
                        };

                        Ok([message_word(0)?, message_word(1)?])
                    },
                )?;

                let poseidon_hasher = PoseidonHash::init(
                    config.poseidon_chip(),
                    layouter.namespace(|| "Poseidon init"),
                    ConstantLength::<2>,
                )?;
                let poseidon_output = poseidon_hasher.hash(
                    layouter.namespace(|| "Poseidon hash (nk, rho_old)"),
                    poseidon_message,
                )?;
                let poseidon_output: CellValue<pallas::Base> = poseidon_output.inner.into();
                poseidon_output
            };

            // Add hash output to psi using standard PLONK
            // `scalar` = poseidon_hash(nk, rho_old) + psi_old.
            //
            let scalar = {
                let scalar_val = nk_rho_old
                    .value()
                    .zip(psi_old.value())
                    .map(|(nk_rho_old, psi_old)| nk_rho_old + psi_old);
                let scalar = self.load_private(
                    layouter.namespace(|| "poseidon_hash(nk, rho_old) + psi_old"),
                    config.advices[0],
                    scalar_val,
                )?;

                config.plonk_chip().add(
                    layouter.namespace(|| "poseidon_hash(nk, rho_old) + psi_old"),
                    nk_rho_old,
                    psi_old,
                    scalar,
                    Some(pallas::Base::one()),
                    Some(pallas::Base::one()),
                    Some(pallas::Base::one()),
                )?;

                scalar
            };

            // Multiply scalar by NullifierK
            // `product` = [poseidon_hash(nk, rho_old) + psi_old] NullifierK.
            //
            let product = {
                let nullifier_k = FixedPointBaseField::from_inner(ecc_chip.clone(), NullifierK);
                nullifier_k.mul(
                    layouter.namespace(|| "[poseidon_output + psi_old] NullifierK"),
                    scalar,
                )?
            };

            // Add cm_old to multiplied fixed base to get nf_old
            // cm_old + [poseidon_output + psi_old] NullifierK
            let nf_old = cm_old
                .add(layouter.namespace(|| "nf_old"), &product)?
                .extract_p();

            // Constrain nf_old to equal public input
            layouter.constrain_instance(nf_old.inner().cell(), config.primary, NF_OLD)?;

            nf_old
        };

        // Spend authority
        {
            // alpha_commitment = [alpha] SpendAuthG
            let (alpha_commitment, _) = {
                let spend_auth_g = OrchardFixedBasesFull::SpendAuthG;
                let spend_auth_g = FixedPoint::from_inner(ecc_chip.clone(), spend_auth_g);
                spend_auth_g.mul(layouter.namespace(|| "[alpha] SpendAuthG"), self.alpha)?
            };

            // [alpha] SpendAuthG + ak
            let rk = alpha_commitment.add(layouter.namespace(|| "rk"), &ak)?;

            // Constrain rk to equal public input
            layouter.constrain_instance(rk.inner().x().cell(), config.primary, RK_X)?;
            layouter.constrain_instance(rk.inner().y().cell(), config.primary, RK_Y)?;
        }

        // Diversified address integrity.
        let pk_d_old = {
            let commit_ivk_config = config.commit_ivk_config.clone();

            let ivk = {
                let rivk = self.rivk.map(|rivk| *rivk);

                commit_ivk_config.assign_region(
                    config.sinsemilla_chip_1(),
                    ecc_chip.clone(),
                    layouter.namespace(|| "CommitIvk"),
                    *ak.extract_p().inner(),
                    nk,
                    rivk,
                )?
            };

            // [ivk] g_d_old
            // The scalar value is passed through and discarded.
            let (derived_pk_d_old, _ivk) =
                g_d_old.mul(layouter.namespace(|| "[ivk] g_d_old"), ivk.inner())?;

            // Constrain derived pk_d_old to equal witnessed pk_d_old
            // This addresses the case where ivk = ⊥ , since ⊥ maps to 0 and variable-base
            // scalar multiplication maps [0]B to (0, 0).
            let pk_d_old = Point::new(
                ecc_chip.clone(),
                layouter.namespace(|| "witness pk_d_old"),
                self.pk_d_old.map(|pk_d_old| (*pk_d_old).to_affine()),
            )?;
            derived_pk_d_old
                .constrain_equal(layouter.namespace(|| "pk_d_old equality"), &pk_d_old)?;

            pk_d_old
        };

        // Old note commitment integrity.
        {
            let old_note_commit_config = config.old_note_commit_config.clone();

            let rcm_old = self.rcm_old.as_ref().map(|rcm_old| **rcm_old);

            // g★_d || pk★_d || i2lebsp_{64}(v) || i2lebsp_{255}(rho) || i2lebsp_{255}(psi)
            let derived_cm_old = old_note_commit_config.assign_region(
                layouter.namespace(|| {
                    "g★_d || pk★_d || i2lebsp_{64}(v) || i2lebsp_{255}(rho) || i2lebsp_{255}(psi)"
                }),
                config.sinsemilla_chip_1(),
                config.ecc_chip(),
                g_d_old.inner(),
                pk_d_old.inner(),
                v_old,
                rho_old,
                psi_old,
                rcm_old,
            )?;

            // Constrain derived cm_old to equal witnessed cm_old
            derived_cm_old.constrain_equal(layouter.namespace(|| "cm_old equality"), &cm_old)?;
        }

        // New note commitment integrity.
        {
            let new_note_commit_config = config.new_note_commit_config.clone();

            // Witness g_d_new_star
            let g_d_new = {
                let g_d_new = self
                    .g_d_new_star
                    .map(|bytes| pallas::Affine::from_bytes(&bytes).unwrap());
                Point::new(
                    ecc_chip.clone(),
                    layouter.namespace(|| "witness g_d_new_star"),
                    g_d_new,
                )?
            };

            // Witness pk_d_new_star
            let pk_d_new = {
                let pk_d_new = self
                    .pk_d_new_star
                    .map(|bytes| pallas::Affine::from_bytes(&bytes).unwrap());
                Point::new(
                    ecc_chip,
                    layouter.namespace(|| "witness pk_d_new"),
                    pk_d_new,
                )?
            };

            // Witness psi_new
            let psi_new = self.load_private(
                layouter.namespace(|| "witness psi_new"),
                config.advices[0],
                self.psi_new,
            )?;

            let rcm_new = self.rcm_new.as_ref().map(|rcm_new| **rcm_new);

            // g★_d || pk★_d || i2lebsp_{64}(v) || i2lebsp_{255}(rho) || i2lebsp_{255}(psi)
            let cm_new = new_note_commit_config.assign_region(
                layouter.namespace(|| {
                    "g★_d || pk★_d || i2lebsp_{64}(v) || i2lebsp_{255}(rho) || i2lebsp_{255}(psi)"
                }),
                config.sinsemilla_chip_2(),
                config.ecc_chip(),
                g_d_new.inner(),
                pk_d_new.inner(),
                v_new,
                *nf_old.inner(),
                psi_new,
                rcm_new,
            )?;

            let cmx = cm_new.extract_p();

            // Constrain cmx to equal public input
            layouter.constrain_instance(cmx.inner().cell(), config.primary, CMX)?;
        }

        // Constrain v_old - v_new = magnitude * sign
        // Either v_old = 0, or anchor equals public input
        layouter.assign_region(
            || "v_old - v_new = magnitude * sign",
            |mut region| {
                copy(&mut region, || "v_old", config.advices[0], 0, &v_old)?;
                copy(&mut region, || "v_new", config.advices[1], 0, &v_new)?;
                let (magnitude, sign) = v_net;
                copy(
                    &mut region,
                    || "v_net magnitude",
                    config.advices[2],
                    0,
                    &magnitude,
                )?;
                copy(&mut region, || "v_net sign", config.advices[3], 0, &sign)?;

                copy(&mut region, || "anchor", config.advices[4], 0, &anchor)?;
                region.assign_advice_from_instance(
                    || "pub input anchor",
                    config.primary,
                    ANCHOR,
                    config.advices[5],
                    0,
                )?;

                region.assign_advice_from_instance(
                    || "enable spends",
                    config.primary,
                    ENABLE_SPEND,
                    config.advices[6],
                    0,
                )?;

                region.assign_advice_from_instance(
                    || "enable outputs",
                    config.primary,
                    ENABLE_OUTPUT,
                    config.advices[7],
                    0,
                )?;

                config.q_orchard.enable(&mut region, 0)
            },
        )?;

        Ok(())
    }
}

/// The verifying key for the Orchard Action circuit.
#[derive(Debug)]
pub struct VerifyingKey {
    params: halo2::poly::commitment::Params<vesta::Affine>,
    vk: plonk::VerifyingKey<vesta::Affine>,
}

impl VerifyingKey {
    /// Builds the verifying key.
    pub fn build() -> Self {
        let params = halo2::poly::commitment::Params::new(K);
        let circuit: Circuit = Default::default();

        let vk = plonk::keygen_vk(&params, &circuit).unwrap();

        VerifyingKey { params, vk }
    }
}

/// The proving key for the Orchard Action circuit.
#[derive(Debug)]
pub struct ProvingKey {
    params: halo2::poly::commitment::Params<vesta::Affine>,
    pk: plonk::ProvingKey<vesta::Affine>,
}

impl ProvingKey {
    /// Builds the proving key.
    pub fn build() -> Self {
        let params = halo2::poly::commitment::Params::new(K);
        let circuit: Circuit = Default::default();

        let vk = plonk::keygen_vk(&params, &circuit).unwrap();
        let pk = plonk::keygen_pk(&params, vk, &circuit).unwrap();

        ProvingKey { params, pk }
    }
}

/// Public inputs to the Orchard Action circuit.
#[derive(Debug)]
pub struct Instance {
    pub(crate) anchor: Anchor,
    pub(crate) cv_net: ValueCommitment,
    pub(crate) nf_old: Nullifier,
    pub(crate) rk: VerificationKey<SpendAuth>,
    pub(crate) cmx: ExtractedNoteCommitment,
    pub(crate) enable_spend: bool,
    pub(crate) enable_output: bool,
}

impl Instance {
    fn to_halo2_instance(&self) -> [[vesta::Scalar; 9]; 1] {
        let mut instance = [vesta::Scalar::zero(); 9];

        instance[ANCHOR] = *self.anchor;
        instance[CV_NET_X] = self.cv_net.x();
        instance[CV_NET_Y] = self.cv_net.y();
        instance[NF_OLD] = *self.nf_old;

        let rk = pallas::Point::from_bytes(&self.rk.clone().into())
            .unwrap()
            .to_affine()
            .coordinates()
            .unwrap();

        instance[RK_X] = *rk.x();
        instance[RK_Y] = *rk.y();
        instance[CMX] = *self.cmx;
        instance[ENABLE_SPEND] = vesta::Scalar::from_u64(self.enable_spend.into());
        instance[ENABLE_OUTPUT] = vesta::Scalar::from_u64(self.enable_output.into());

        [instance]
    }
}

/// A proof of the validity of an Orchard [`Bundle`].
///
/// [`Bundle`]: crate::bundle::Bundle
#[derive(Debug, Clone)]
pub struct Proof(Vec<u8>);

impl AsRef<[u8]> for Proof {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl Proof {
    /// Returns the amount of heap-allocated memory used by this proof.
    pub(crate) fn dynamic_usage(&self) -> usize {
        self.0.capacity() * mem::size_of::<u8>()
    }

    /// Creates a proof for the given circuits and instances.
    pub fn create(
        pk: &ProvingKey,
        circuits: &[Circuit],
        instances: &[Instance],
    ) -> Result<Self, plonk::Error> {
        let instances: Vec<_> = instances.iter().map(|i| i.to_halo2_instance()).collect();
        let instances: Vec<Vec<_>> = instances
            .iter()
            .map(|i| i.iter().map(|c| &c[..]).collect())
            .collect();
        let instances: Vec<_> = instances.iter().map(|i| &i[..]).collect();

        let mut transcript = Blake2bWrite::<_, vesta::Affine, _>::init(vec![]);
        plonk::create_proof(&pk.params, &pk.pk, circuits, &instances, &mut transcript)?;
        Ok(Proof(transcript.finalize()))
    }

    /// Verifies this proof with the given instances.
    pub fn verify(&self, vk: &VerifyingKey, instances: &[Instance]) -> Result<(), plonk::Error> {
        let instances: Vec<_> = instances.iter().map(|i| i.to_halo2_instance()).collect();
        let instances: Vec<Vec<_>> = instances
            .iter()
            .map(|i| i.iter().map(|c| &c[..]).collect())
            .collect();
        let instances: Vec<_> = instances.iter().map(|i| &i[..]).collect();

        let msm = vk.params.empty_msm();
        let mut transcript = Blake2bRead::init(&self.0[..]);
        let guard = plonk::verify_proof(&vk.params, &vk.vk, msm, &instances, &mut transcript)?;
        let msm = guard.clone().use_challenges();
        if msm.eval() {
            Ok(())
        } else {
            Err(plonk::Error::ConstraintSystemFailure)
        }
    }

    /// Constructs a new Proof value.
    pub fn new(bytes: Vec<u8>) -> Self {
        Proof(bytes)
    }
}

#[cfg(test)]
mod tests {
    use ff::Field;
    use group::GroupEncoding;
    use halo2::dev::MockProver;
    use pasta_curves::pallas;
    use rand::rngs::OsRng;
    use std::iter;

    use super::{Circuit, Instance, Proof, ProvingKey, VerifyingKey, K};
    use crate::{
        keys::SpendValidatingKey,
        note::Note,
        tree::MerklePath,
        value::{ValueCommitTrapdoor, ValueCommitment},
    };

    // TODO: recast as a proptest
    #[test]
    fn round_trip() {
        let mut rng = OsRng;

        let (circuits, instances): (Vec<_>, Vec<_>) = iter::once(())
            .map(|()| {
                let (_, fvk, spent_note) = Note::dummy(&mut rng, None);

                let sender_address = fvk.default_address();
                let nk = *fvk.nk();
                let rivk = *fvk.rivk();
                let nf_old = spent_note.nullifier(&fvk);
                let ak: SpendValidatingKey = fvk.into();
                let alpha = pallas::Scalar::random(&mut rng);
                let rk = ak.randomize(&alpha);

                let (_, _, output_note) = Note::dummy(&mut rng, Some(nf_old));
                let cmx = output_note.commitment().into();

                let value = spent_note.value() - output_note.value();
                let cv_net = ValueCommitment::derive(value.unwrap(), ValueCommitTrapdoor::zero());

                let path = MerklePath::dummy(&mut rng);
                let anchor = path.root(spent_note.commitment().into()).unwrap();

                (
                    Circuit {
                        path: Some(path.auth_path()),
                        pos: Some(path.position()),
                        g_d_old: Some(sender_address.g_d()),
                        pk_d_old: Some(*sender_address.pk_d()),
                        v_old: Some(spent_note.value()),
                        rho_old: Some(spent_note.rho()),
                        psi_old: Some(spent_note.rseed().psi(&spent_note.rho())),
                        rcm_old: Some(spent_note.rseed().rcm(&spent_note.rho())),
                        cm_old: Some(spent_note.commitment()),
                        alpha: Some(alpha),
                        ak: Some(ak),
                        nk: Some(nk),
                        rivk: Some(rivk),
                        g_d_new_star: Some((*output_note.recipient().g_d()).to_bytes()),
                        pk_d_new_star: Some(output_note.recipient().pk_d().to_bytes()),
                        v_new: Some(output_note.value()),
                        psi_new: Some(output_note.rseed().psi(&output_note.rho())),
                        rcm_new: Some(output_note.rseed().rcm(&output_note.rho())),
                        rcv: Some(ValueCommitTrapdoor::zero()),
                    },
                    Instance {
                        anchor,
                        cv_net,
                        nf_old,
                        rk,
                        cmx,
                        enable_spend: true,
                        enable_output: true,
                    },
                )
            })
            .unzip();

        let vk = VerifyingKey::build();
        for (circuit, instance) in circuits.iter().zip(instances.iter()) {
            assert_eq!(
                MockProver::run(
                    K,
                    circuit,
                    instance
                        .to_halo2_instance()
                        .iter()
                        .map(|p| p.to_vec())
                        .collect()
                )
                .unwrap()
                .verify(),
                Ok(())
            );
        }

        let pk = ProvingKey::build();
        let proof = Proof::create(&pk, &circuits, &instances).unwrap();
        assert!(proof.verify(&vk, &instances).is_ok());
    }

    #[cfg(feature = "dev-graph")]
    #[test]
    fn print_action_circuit() {
        use plotters::prelude::*;

        let root = BitMapBackend::new("action-circuit-layout.png", (1024, 768)).into_drawing_area();
        root.fill(&WHITE).unwrap();
        let root = root
            .titled("Orchard Action Circuit", ("sans-serif", 60))
            .unwrap();

        let circuit = Circuit {
            path: None,
            pos: None,
            g_d_old: None,
            pk_d_old: None,
            v_old: None,
            rho_old: None,
            psi_old: None,
            rcm_old: None,
            cm_old: None,
            alpha: None,
            ak: None,
            nk: None,
            rivk: None,
            g_d_new_star: None,
            pk_d_new_star: None,
            v_new: None,
            psi_new: None,
            rcm_new: None,
            rcv: None,
        };
        halo2::dev::CircuitLayout::default()
            .show_labels(false)
            .view_height(0..(1 << 11))
            .render(&circuit, &root)
            .unwrap();
    }
}

//! The Orchard Action circuit implementation.

use std::mem;

use group::Curve;
use halo2::{
    circuit::{Layouter, SimpleFloorPlanner},
    plonk::{self, Advice, Column, Instance as InstanceColumn, Selector},
    poly::Rotation,
    transcript::{Blake2bRead, Blake2bWrite},
};
use pasta_curves::{arithmetic::FieldExt, pallas, vesta};

use crate::{
    constants::MERKLE_DEPTH_ORCHARD,
    keys::{
        CommitIvkRandomness, DiversifiedTransmissionKey, NullifierDerivingKey, SpendValidatingKey,
    },
    note::{
        commitment::{NoteCommitTrapdoor, NoteCommitment},
        nullifier::Nullifier,
        ExtractedNoteCommitment,
    },
    primitives::{
        poseidon,
        redpallas::{SpendAuth, VerificationKey},
    },
    spec::NonIdentityPallasPoint,
    tree::Anchor,
    value::{NoteValue, ValueCommitTrapdoor, ValueCommitment},
};
use gadget::{
    ecc::{
        chip::{EccChip, EccConfig},
        Point,
    },
    poseidon::{Pow5T3Chip as PoseidonChip, Pow5T3Config as PoseidonConfig},
    sinsemilla::{
        chip::{SinsemillaChip, SinsemillaConfig},
        merkle::chip::{MerkleChip, MerkleConfig},
    },
    utilities::{
        enable_flag::{EnableFlagChip, EnableFlagConfig},
        plonk::{PLONKChip, PLONKConfig},
        CellValue, UtilitiesInstructions,
    },
};

use std::convert::TryInto;

pub(crate) mod gadget;

/// Size of the Orchard circuit.
const K: u32 = 11;

/// Configuration needed to use the Orchard Action circuit.
#[derive(Clone, Debug)]
pub struct Config {
    q_primary: Selector,
    primary: Column<InstanceColumn>,
    advices: [Column<Advice>; 10],
    enable_flag_config: EnableFlagConfig,
    ecc_config: EccConfig,
    poseidon_config: PoseidonConfig<pallas::Base>,
    plonk_config: PLONKConfig,
    merkle_config_1: MerkleConfig,
    merkle_config_2: MerkleConfig,
    sinsemilla_config_1: SinsemillaConfig,
    sinsemilla_config_2: SinsemillaConfig,
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

        // Fixed columns for the Sinsemilla generator lookup table
        let table_idx = meta.fixed_column();
        let lookup = (table_idx, meta.fixed_column(), meta.fixed_column());

        // Shared fixed column used to load constants.
        // TODO: Replace with public inputs API
        let ecc_constants = [meta.fixed_column(), meta.fixed_column()];
        let sinsemilla_1_constants = [
            meta.fixed_column(),
            meta.fixed_column(),
            meta.fixed_column(),
            meta.fixed_column(),
            meta.fixed_column(),
            meta.fixed_column(),
        ];
        let sinsemilla_2_constants = [
            meta.fixed_column(),
            meta.fixed_column(),
            meta.fixed_column(),
            meta.fixed_column(),
            meta.fixed_column(),
            meta.fixed_column(),
        ];

        // Permutation over all advice columns and `constants` columns.
        // TODO: Replace `*_constants` with public inputs API.
        for advice in advices.iter() {
            meta.enable_equality((*advice).into());
        }
        for fixed in ecc_constants.iter() {
            meta.enable_equality((*fixed).into());
        }
        for fixed in sinsemilla_1_constants.iter() {
            meta.enable_equality((*fixed).into());
        }
        for fixed in sinsemilla_2_constants.iter() {
            meta.enable_equality((*fixed).into());
        }

        // Configuration for `enable_spends` and `enable_outputs` flags logic
        // TODO: this may change with public inputs API.
        let enable_flag_config = EnableFlagChip::configure(meta, [advices[0], advices[1]]);

        // Configuration for curve point operations.
        // This uses 10 advice columns and spans the whole circuit.
        let ecc_config = EccChip::configure(meta, advices, table_idx, ecc_constants);

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
            let sinsemilla_config_1 = SinsemillaChip::configure(
                meta,
                advices[..5].try_into().unwrap(),
                lookup,
                sinsemilla_1_constants,
            );
            let merkle_config_1 = MerkleChip::configure(meta, sinsemilla_config_1.clone());

            (sinsemilla_config_1, merkle_config_1)
        };

        // Configuration for a Sinsemilla hash instantiation and a
        // Merkle hash instantiation using this Sinsemilla instance.
        // Since the Sinsemilla config uses only 5 advice columns,
        // we can fit two instances side-by-side.
        let (sinsemilla_config_2, merkle_config_2) = {
            let sinsemilla_config_2 = SinsemillaChip::configure(
                meta,
                advices[5..].try_into().unwrap(),
                lookup,
                sinsemilla_2_constants,
            );
            let merkle_config_2 = MerkleChip::configure(meta, sinsemilla_config_2.clone());

            (sinsemilla_config_2, merkle_config_2)
        };

        // TODO: Infrastructure to handle public inputs.
        let q_primary = meta.selector();
        let primary = meta.instance_column();

        // Placeholder gate so there is something for the prover to operate on.
        // We need a selector so that the gate is disabled by default, and doesn't
        // interfere with the blinding factors.
        let advice = meta.advice_column();
        let selector = meta.selector();

        meta.create_gate("TODO", |meta| {
            let a = meta.query_advice(advice, Rotation::cur());
            let s = meta.query_selector(selector);

            vec![s * a]
        });

        Config {
            q_primary,
            primary,
            advices,
            enable_flag_config,
            ecc_config,
            poseidon_config,
            plonk_config,
            merkle_config_1,
            merkle_config_2,
            sinsemilla_config_1,
            sinsemilla_config_2,
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
        let circuit: Circuit = Default::default(); // TODO

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
        let circuit: Circuit = Default::default(); // TODO

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
    fn to_halo2_instance(&self) -> [[vesta::Scalar; 0]; 1] {
        // TODO
        [[]]
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
}

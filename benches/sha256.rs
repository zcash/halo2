#[macro_use]
extern crate criterion;

extern crate halo2;
use halo2::{
    arithmetic::FieldExt,
    circuit::Chip,
    circuit::{layouter, Cell, Layouter, Permutation, Region},
    gadget::sha256::{BlockWord, Sha256, Table16Chip, Table16Config, BLOCK_SIZE},
    pasta::{EqAffine, Fq},
    plonk::*,
    plonk::{Advice, Assignment, Circuit, Column, ConstraintSystem, Error, Fixed},
    poly::commitment::Params,
    transcript::{DummyHashRead, DummyHashWrite},
};

use std::{
    cmp,
    collections::HashMap,
    fmt,
    fs::File,
    io::{prelude::*, BufReader},
    marker::PhantomData,
    path::Path,
};

use criterion::Criterion;

fn bench(name: &str, k: u32, c: &mut Criterion) {
    /// This represents an advice column at a certain row in the ConstraintSystem
    #[derive(Copy, Clone, Debug)]
    pub struct Variable(Column<Advice>, usize);

    struct MyCircuit {}

    struct MyLayouter<'a, F: FieldExt, CS: Assignment<F> + 'a> {
        cs: &'a mut CS,
        config: Table16Config,
        regions: Vec<usize>,
        /// Stores the first empty row for each column.
        columns: HashMap<Column<Any>, usize>,
        _marker: PhantomData<F>,
    }

    impl<'a, F: FieldExt, CS: Assignment<F> + 'a> fmt::Debug for MyLayouter<'a, F, CS> {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.debug_struct("MyLayouter")
                .field("config", &self.config)
                .field("regions", &self.regions)
                .field("columns", &self.columns)
                .finish()
        }
    }

    impl<'a, FF: FieldExt, CS: Assignment<FF>> MyLayouter<'a, FF, CS> {
        fn new(cs: &'a mut CS, config: Table16Config) -> Result<Self, Error> {
            let mut res = MyLayouter {
                cs,
                config,
                regions: vec![],
                columns: HashMap::default(),
                _marker: PhantomData,
            };

            Table16Chip::load(&mut res)?;

            Ok(res)
        }
    }

    impl<'a, F: FieldExt, CS: Assignment<F> + 'a> Layouter<Table16Chip<F>> for MyLayouter<'a, F, CS> {
        fn config(&self) -> &Table16Config {
            &self.config
        }

        fn assign_region(
            &mut self,
            mut assignment: impl FnMut(Region<'_, Table16Chip<F>>) -> Result<(), Error>,
        ) -> Result<(), Error> {
            let region_index = self.regions.len();

            // Get shape of the region.
            let mut shape = layouter::RegionShape::new(region_index);
            {
                let region: &mut dyn layouter::RegionLayouter<Table16Chip<F>> = &mut shape;
                assignment(region.into())?;
            }

            // Lay out this region. We implement the simplest approach here: position the
            // region starting at the earliest row for which none of the columns are in use.
            let mut region_start = 0;
            for column in shape.columns() {
                region_start =
                    cmp::max(region_start, self.columns.get(column).cloned().unwrap_or(0));
            }
            self.regions.push(region_start);

            // Update column usage information.
            for column in shape.columns() {
                self.columns
                    .insert(*column, region_start + shape.row_count());
            }

            let mut region = MyRegion::new(self, region_index);
            {
                let region: &mut dyn layouter::RegionLayouter<Table16Chip<F>> = &mut region;
                assignment(region.into())?;
            }

            Ok(())
        }
    }

    struct MyRegion<'r, 'a, F: FieldExt, CS: Assignment<F> + 'a> {
        layouter: &'r mut MyLayouter<'a, F, CS>,
        region_index: usize,
        _marker: PhantomData<F>,
    }

    impl<'r, 'a, F: FieldExt, CS: Assignment<F> + 'a> fmt::Debug for MyRegion<'r, 'a, F, CS> {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.debug_struct("MyRegion")
                .field("layouter", &self.layouter)
                .field("region_index", &self.region_index)
                .finish()
        }
    }

    impl<'r, 'a, F: FieldExt, CS: Assignment<F> + 'a> MyRegion<'r, 'a, F, CS> {
        fn new(layouter: &'r mut MyLayouter<'a, F, CS>, region_index: usize) -> Self {
            MyRegion {
                layouter,
                region_index,
                _marker: PhantomData::default(),
            }
        }
    }

    impl<'r, 'a, F: FieldExt, CS: Assignment<F> + 'a> layouter::RegionLayouter<Table16Chip<F>>
        for MyRegion<'r, 'a, F, CS>
    {
        fn assign_advice<'v>(
            &'v mut self,
            column: Column<Advice>,
            offset: usize,
            to: &'v mut (dyn FnMut() -> Result<F, Error> + 'v),
        ) -> Result<Cell, Error> {
            self.layouter.cs.assign_advice(
                column,
                self.layouter.regions[self.region_index] + offset,
                to,
            )?;

            Ok(Cell {
                region_index: self.region_index,
                row_offset: offset,
                column: column.into(),
            })
        }

        fn assign_fixed<'v>(
            &'v mut self,
            column: Column<Fixed>,
            offset: usize,
            to: &'v mut (dyn FnMut() -> Result<F, Error> + 'v),
        ) -> Result<Cell, Error> {
            self.layouter.cs.assign_fixed(
                column,
                self.layouter.regions[self.region_index] + offset,
                to,
            )?;

            Ok(Cell {
                region_index: self.region_index,
                row_offset: offset,
                column: column.into(),
            })
        }

        fn constrain_equal(
            &mut self,
            permutation: &Permutation,
            left: Cell,
            right: Cell,
        ) -> Result<(), Error> {
            let left_column = permutation
                .mapping
                .iter()
                .position(|c| c == &left.column)
                .ok_or(Error::SynthesisError)?;
            let right_column = permutation
                .mapping
                .iter()
                .position(|c| c == &right.column)
                .ok_or(Error::SynthesisError)?;

            self.layouter.cs.copy(
                permutation.index,
                left_column,
                self.layouter.regions[left.region_index] + left.row_offset,
                right_column,
                self.layouter.regions[right.region_index] + right.row_offset,
            )?;

            Ok(())
        }
    }

    impl<F: FieldExt> Circuit<F> for MyCircuit {
        type Config = Table16Config;

        fn configure(meta: &mut ConstraintSystem<F>) -> Table16Config {
            Table16Chip::configure(meta)
        }

        fn synthesize(
            &self,
            cs: &mut impl Assignment<F>,
            config: Table16Config,
        ) -> Result<(), Error> {
            let mut layouter = MyLayouter::new(cs, config)?;

            // Test vector: "abc"
            let test_input = [
                BlockWord::new(0b01100001011000100110001110000000),
                BlockWord::new(0),
                BlockWord::new(0),
                BlockWord::new(0),
                BlockWord::new(0),
                BlockWord::new(0),
                BlockWord::new(0),
                BlockWord::new(0),
                BlockWord::new(0),
                BlockWord::new(0),
                BlockWord::new(0),
                BlockWord::new(0),
                BlockWord::new(0),
                BlockWord::new(0),
                BlockWord::new(0),
            ];

            // Create a message of length 31 blocks
            let mut input = Vec::with_capacity(31 * BLOCK_SIZE);
            for _ in 0..31 {
                input.extend_from_slice(&test_input);
            }

            Sha256::digest(&mut layouter, &input)?;

            Ok(())
        }
    }

    // Initialize the polynomial commitment parameters
    let params_path = Path::new("./benches/sha256_assets/sha256_params");
    if let Err(_) = File::open(&params_path) {
        let params: Params<EqAffine> = Params::new(k);
        let mut buf = Vec::new();

        params.write(&mut buf).expect("Failed to write params");
        let mut file = File::create(&params_path).expect("Failed to create sha256_params");

        file.write_all(&buf[..])
            .expect("Failed to write params to file");
    }

    let params_fs = File::open(&params_path).expect("couldn't load sha256_params");
    let params: Params<EqAffine> =
        Params::read::<_>(&mut BufReader::new(params_fs)).expect("Failed to read params");

    let empty_circuit: MyCircuit = MyCircuit {};

    // Initialize the proving key
    let vk_path = Path::new("./benches/sha256_assets/sha256_vk");
    if let Err(_) = File::open(&vk_path) {
        let vk = keygen_vk(&params, &empty_circuit).expect("keygen_vk should not fail");
        let mut buf = Vec::new();

        vk.write(&mut buf).expect("Failed to write vk");
        let mut file = File::create(&vk_path).expect("Failed to create sha256_vk");

        file.write_all(&buf[..])
            .expect("Failed to write vk to file");
    }

    let vk_fs = File::open(&vk_path).expect("couldn't load sha256_params");
    let vk: VerifyingKey<EqAffine> =
        VerifyingKey::<EqAffine>::read::<_, MyCircuit>(&mut BufReader::new(vk_fs), &params)
            .expect("Failed to read vk");

    let pk = keygen_pk(&params, vk, &empty_circuit).expect("keygen_pk should not fail");

    let circuit: MyCircuit = MyCircuit {};

    let prover_name = name.to_string() + "-prover";
    let verifier_name = name.to_string() + "-verifier";

    // /// Benchmark proof creation
    // c.bench_function(&prover_name, |b| {
    //     b.iter(|| {
    //         let mut transcript = DummyHashWrite::init(vec![], Fq::one());
    //         create_proof(&params, &pk, &circuit, &[], &mut transcript)
    //             .expect("proof generation should not fail");
    //         let proof: Vec<u8> = transcript.finalize();
    //     });
    // });

    // Create a proof
    let proof_path = Path::new("./benches/sha256_assets/sha256_proof");
    if let Err(_) = File::open(&proof_path) {
        let mut transcript = DummyHashWrite::init(vec![], Fq::one());
        create_proof(&params, &pk, &circuit, &[], &mut transcript)
            .expect("proof generation should not fail");
        let proof: Vec<u8> = transcript.finalize();
        let mut file = File::create(&proof_path).expect("Failed to create sha256_proof");
        file.write_all(&proof[..]).expect("Failed to write proof");
    }

    let mut proof_fs = File::open(&proof_path).expect("couldn't load sha256_proof");
    let mut proof = Vec::<u8>::new();
    proof_fs
        .read_to_end(&mut proof)
        .expect("Couldn't read proof");

    c.bench_function(&verifier_name, |b| {
        b.iter(|| {
            let msm = params.empty_msm();
            let mut transcript = DummyHashRead::init(&proof[..], Fq::one());
            let guard = verify_proof(&params, pk.get_vk(), msm, &[], &mut transcript).unwrap();
            let msm = guard.clone().use_challenges();
            assert!(msm.eval());
        });
    });
}

fn criterion_benchmark(c: &mut Criterion) {
    bench("sha256", 16, c);
    // bench("sha256", 20, c);
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);

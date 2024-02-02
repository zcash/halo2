#[cfg(test)]
mod tests {
    use halo2curves::pasta::Fp;

    use crate::{
        circuit::{Layouter, SimpleFloorPlanner},
        dev::MockProver,
    };
    use halo2_common::circuit::Value;
    use halo2_common::plonk::{Circuit, ConstraintSystem, Error, TableColumn};
    use halo2_middleware::poly::Rotation;

    #[test]
    fn table_no_default() {
        const K: u32 = 4;

        #[derive(Clone)]
        struct FaultyCircuitConfig {
            table: TableColumn,
        }

        struct FaultyCircuit;

        impl Circuit<Fp> for FaultyCircuit {
            type Config = FaultyCircuitConfig;
            type FloorPlanner = SimpleFloorPlanner;
            #[cfg(feature = "circuit-params")]
            type Params = ();

            fn without_witnesses(&self) -> Self {
                Self
            }

            fn configure(meta: &mut ConstraintSystem<Fp>) -> Self::Config {
                let a = meta.advice_column();
                let table = meta.lookup_table_column();

                meta.lookup("", |cells| {
                    let a = cells.query_advice(a, Rotation::cur());
                    vec![(a, table)]
                });

                Self::Config { table }
            }

            fn synthesize(
                &self,
                config: Self::Config,
                mut layouter: impl Layouter<Fp>,
            ) -> Result<(), Error> {
                layouter.assign_table(
                    || "duplicate assignment",
                    |mut table| {
                        table.assign_cell(
                            || "default",
                            config.table,
                            1,
                            || Value::known(Fp::zero()),
                        )
                    },
                )
            }
        }

        let prover = MockProver::run(K, &FaultyCircuit, vec![]);
        assert_eq!(
            format!("{}", prover.unwrap_err()),
            "TableColumn { inner: Column { index: 0, column_type: Fixed } } not fully assigned. Help: assign a value at offset 0."
        );
    }

    #[test]
    fn table_overwrite_default() {
        const K: u32 = 4;

        #[derive(Clone)]
        struct FaultyCircuitConfig {
            table: TableColumn,
        }

        struct FaultyCircuit;

        impl Circuit<Fp> for FaultyCircuit {
            type Config = FaultyCircuitConfig;
            type FloorPlanner = SimpleFloorPlanner;
            #[cfg(feature = "circuit-params")]
            type Params = ();

            fn without_witnesses(&self) -> Self {
                Self
            }

            fn configure(meta: &mut ConstraintSystem<Fp>) -> Self::Config {
                let a = meta.advice_column();
                let table = meta.lookup_table_column();

                meta.lookup("", |cells| {
                    let a = cells.query_advice(a, Rotation::cur());
                    vec![(a, table)]
                });

                Self::Config { table }
            }

            fn synthesize(
                &self,
                config: Self::Config,
                mut layouter: impl Layouter<Fp>,
            ) -> Result<(), Error> {
                layouter.assign_table(
                    || "duplicate assignment",
                    |mut table| {
                        table.assign_cell(
                            || "default",
                            config.table,
                            0,
                            || Value::known(Fp::zero()),
                        )?;
                        table.assign_cell(
                            || "duplicate",
                            config.table,
                            0,
                            || Value::known(Fp::zero()),
                        )
                    },
                )
            }
        }

        let prover = MockProver::run(K, &FaultyCircuit, vec![]);
        assert_eq!(
            format!("{}", prover.unwrap_err()),
            "Attempted to overwrite default value Value { inner: Some(Trivial(0x0000000000000000000000000000000000000000000000000000000000000000)) } with Value { inner: Some(Trivial(0x0000000000000000000000000000000000000000000000000000000000000000)) } in TableColumn { inner: Column { index: 0, column_type: Fixed } }"
        );
    }

    #[test]
    fn table_reuse_column() {
        const K: u32 = 4;

        #[derive(Clone)]
        struct FaultyCircuitConfig {
            table: TableColumn,
        }

        struct FaultyCircuit;

        impl Circuit<Fp> for FaultyCircuit {
            type Config = FaultyCircuitConfig;
            type FloorPlanner = SimpleFloorPlanner;
            #[cfg(feature = "circuit-params")]
            type Params = ();

            fn without_witnesses(&self) -> Self {
                Self
            }

            fn configure(meta: &mut ConstraintSystem<Fp>) -> Self::Config {
                let a = meta.advice_column();
                let table = meta.lookup_table_column();

                meta.lookup("", |cells| {
                    let a = cells.query_advice(a, Rotation::cur());
                    vec![(a, table)]
                });

                Self::Config { table }
            }

            fn synthesize(
                &self,
                config: Self::Config,
                mut layouter: impl Layouter<Fp>,
            ) -> Result<(), Error> {
                layouter.assign_table(
                    || "first assignment",
                    |mut table| {
                        table.assign_cell(
                            || "default",
                            config.table,
                            0,
                            || Value::known(Fp::zero()),
                        )
                    },
                )?;

                layouter.assign_table(
                    || "reuse",
                    |mut table| {
                        table.assign_cell(|| "reuse", config.table, 1, || Value::known(Fp::zero()))
                    },
                )
            }
        }

        let prover = MockProver::run(K, &FaultyCircuit, vec![]);
        assert_eq!(
            format!("{}", prover.unwrap_err()),
            "TableColumn { inner: Column { index: 0, column_type: Fixed } } has already been used"
        );
    }

    #[test]
    fn table_uneven_columns() {
        const K: u32 = 4;

        #[derive(Clone)]
        struct FaultyCircuitConfig {
            table: (TableColumn, TableColumn),
        }

        struct FaultyCircuit;

        impl Circuit<Fp> for FaultyCircuit {
            type Config = FaultyCircuitConfig;
            type FloorPlanner = SimpleFloorPlanner;
            #[cfg(feature = "circuit-params")]
            type Params = ();

            fn without_witnesses(&self) -> Self {
                Self
            }

            fn configure(meta: &mut ConstraintSystem<Fp>) -> Self::Config {
                let a = meta.advice_column();
                let table = (meta.lookup_table_column(), meta.lookup_table_column());
                meta.lookup("", |cells| {
                    let a = cells.query_advice(a, Rotation::cur());

                    vec![(a.clone(), table.0), (a, table.1)]
                });

                Self::Config { table }
            }

            fn synthesize(
                &self,
                config: Self::Config,
                mut layouter: impl Layouter<Fp>,
            ) -> Result<(), Error> {
                layouter.assign_table(
                    || "table with uneven columns",
                    |mut table| {
                        table.assign_cell(|| "", config.table.0, 0, || Value::known(Fp::zero()))?;
                        table.assign_cell(|| "", config.table.0, 1, || Value::known(Fp::zero()))?;

                        table.assign_cell(|| "", config.table.1, 0, || Value::known(Fp::zero()))
                    },
                )
            }
        }

        let prover = MockProver::run(K, &FaultyCircuit, vec![]);
        assert_eq!(
            format!("{}", prover.unwrap_err()),
            "TableColumn { inner: Column { index: 0, column_type: Fixed } } has length 2 while TableColumn { inner: Column { index: 1, column_type: Fixed } } has length 1"
        );
    }
}

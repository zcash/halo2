#[test]
fn test_slot_in() {
    use crate::circuit::layouter::RegionShape;
    use halo2_common::circuit::floor_planner::v1::strategy::slot_in;
    use halo2_middleware::circuit::{Any, Column};

    let regions = vec![
        RegionShape {
            region_index: 0.into(),
            columns: vec![Column::new(0, Any::advice()), Column::new(1, Any::advice())]
                .into_iter()
                .map(|a| a.into())
                .collect(),
            row_count: 15,
        },
        RegionShape {
            region_index: 1.into(),
            columns: vec![Column::new(2, Any::advice())]
                .into_iter()
                .map(|a| a.into())
                .collect(),
            row_count: 10,
        },
        RegionShape {
            region_index: 2.into(),
            columns: vec![Column::new(2, Any::advice()), Column::new(0, Any::advice())]
                .into_iter()
                .map(|a| a.into())
                .collect(),
            row_count: 10,
        },
    ];
    assert_eq!(
        slot_in(regions)
            .0
            .into_iter()
            .map(|(i, _)| i)
            .collect::<Vec<_>>(),
        vec![0.into(), 0.into(), 15.into()]
    );
}

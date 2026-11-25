use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;

use ff::Field;
use halo2_poseidon::{Absorbing, ConstantLength, Hash, P128Pow5T3, Squeezing};
use pasta_curves::{pallas::Base as Fp, vesta::Base as Fq};

struct CountingAlloc;

thread_local! {
    static ALLOC_COUNT: Cell<usize> = const { Cell::new(0) };
}

// SAFETY: Delegates directly to `std::alloc::System` while incrementing a thread-local
// allocation counter so concurrent `libtest` harness threads cannot introduce noise.
#[allow(unsafe_code)]
unsafe impl GlobalAlloc for CountingAlloc {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let _ = ALLOC_COUNT.try_with(|count| count.set(count.get().saturating_add(1)));
        unsafe { System.alloc(layout) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe { System.dealloc(ptr, layout) }
    }
}

#[global_allocator]
static GLOBAL: CountingAlloc = CountingAlloc;

fn measure_allocs<R>(f: impl FnOnce() -> R) -> (R, usize) {
    let before = ALLOC_COUNT.with(Cell::get);
    let out = f();
    let after = ALLOC_COUNT.with(Cell::get);
    (out, after - before)
}

#[test]
fn p128pow5t3_hash_performs_zero_heap_allocations() {
    let (_, allocs_fp_2) = measure_allocs(|| {
        Hash::<Fp, P128Pow5T3, ConstantLength<2>, 3, 2>::init().hash([Fp::from(6), Fp::from(42)])
    });
    assert_eq!(
        allocs_fp_2, 0,
        "ConstantLength<2> Fp hash must not allocate"
    );

    let (_, allocs_fq_2) = measure_allocs(|| {
        Hash::<Fq, P128Pow5T3, ConstantLength<2>, 3, 2>::init().hash([Fq::from(6), Fq::from(42)])
    });
    assert_eq!(
        allocs_fq_2, 0,
        "ConstantLength<2> Fq hash must not allocate"
    );

    let (_, allocs_fp_4) = measure_allocs(|| {
        Hash::<Fp, P128Pow5T3, ConstantLength<4>, 3, 2>::init().hash([
            Fp::ONE,
            Fp::from(2),
            Fp::from(3),
            Fp::from(4),
        ])
    });
    assert_eq!(
        allocs_fp_4, 0,
        "Multi-block ConstantLength<4> Fp hash must not allocate"
    );

    let (_, allocs_fp_5) = measure_allocs(|| {
        Hash::<Fp, P128Pow5T3, ConstantLength<5>, 3, 2>::init().hash([
            Fp::ONE,
            Fp::from(2),
            Fp::from(3),
            Fp::from(4),
            Fp::from(5),
        ])
    });
    assert_eq!(
        allocs_fp_5, 0,
        "Multi-block ConstantLength<5> Fp hash must not allocate"
    );

    let (_, allocs_fq_5) = measure_allocs(|| {
        Hash::<Fq, P128Pow5T3, ConstantLength<5>, 3, 2>::init().hash([
            Fq::ONE,
            Fq::from(2),
            Fq::from(3),
            Fq::from(4),
            Fq::from(5),
        ])
    });
    assert_eq!(
        allocs_fq_5, 0,
        "Multi-block ConstantLength<5> Fq hash must not allocate"
    );

    let (squeezed, allocs_modes) = measure_allocs(|| {
        let mut abs = Absorbing::<Fp, 2>::init_empty();
        abs.absorb(Fp::ONE).unwrap();
        let mut sq = Squeezing::<Fp, 2>::init_full([Fp::ONE, Fp::from(2)]);
        (sq.squeeze(), sq.squeeze(), sq.squeeze())
    });
    assert_eq!(squeezed, (Some(Fp::ONE), Some(Fp::from(2)), None));
    assert_eq!(
        allocs_modes, 0,
        "Absorbing::init_empty and Squeezing::init_full must not allocate"
    );
}

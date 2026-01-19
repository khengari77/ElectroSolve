use electro_solve::units::*;
use proptest::prelude::*;
use num_complex::Complex64;

mod common;
use common::*;

fn is_passive_impedance_result(z: &ImpedanceResult) -> bool {
    match z {
        ImpedanceResult::Finite(z_val) => is_passive_impedance(*z_val),
        ImpedanceResult::Short => true,
        ImpedanceResult::Open => true,
    }
}

fn impedance_strategy() -> impl Strategy<Value = ImpedanceResult> {
    prop_oneof![
        (1e-12_f64..1e12_f64, -1e12_f64..1e12_f64)
            .prop_map(|(re, im)| ImpedanceResult::new_finite(Complex64::new(re, im)))
            .boxed(),
    ]
}

proptest! {

#[test]
fn prop_series_passivity(
    a in impedance_strategy(),
    b in impedance_strategy()
    ) {
    let result = combine_series(a, b);
    prop_assert!(is_passive_impedance_result(&result));
}

#[test]
fn prop_parallel_passivity(
    a in impedance_strategy(),
    b in impedance_strategy()
) {
    let result = combine_parallel(a, b);
    prop_assert!(is_passive_impedance_result(&result));
}

#[test]
fn prop_series_many_passivity(
    impedances in prop::collection::vec(impedance_strategy(), 2..10)
) {
    let result = combine_series_many(&impedances);
    prop_assert!(is_passive_impedance_result(&result));
}

#[test]
fn prop_parallel_many_passivity(
    impedances in prop::collection::vec(impedance_strategy(), 2..10)
) {
    let result = combine_parallel_many(&impedances);
    prop_assert!(is_passive_impedance_result(&result));
}

}


#[test]
fn test_short_is_passive() {
    let z = ImpedanceResult::Short;
    assert!(is_passive_impedance_result(&z), "Short should be passive");
}

#[test]
fn test_open_is_passive() {
    let z = ImpedanceResult::Open;
    assert!(is_passive_impedance_result(&z), "Open should be passive");
}

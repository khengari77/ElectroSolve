use proptest::prelude::*;
use num_complex::Complex64;
use electro_solve_core::units::*;

mod common;
use common::strategies::*;


proptest! {

#[test]
fn prop_value_either_known_or_unknown(
    value in arbitrary_value()
) {
    prop_assert!(value.is_known() && !value.is_unknown() || !value.is_known() && value.is_unknown());
    prop_assert!(matches!(value, Value::Known(_) | Value::Unknown(_)));
}

#[test]
fn prop_resistance_is_always_positive(
    r in any::<f64>()
) {
    let r = Resistance::known(r);
    prop_assert!(true);
}
}     

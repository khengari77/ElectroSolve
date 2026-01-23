use approx::assert_relative_eq;
use num_complex::Complex64;
use proptest::prelude::*;
use electro_solve::units::*;

mod common;
use common::*;
use common::strategies::*;

proptest! {

#[test]
fn prop_series_identity_left(z in impedance_strategy()) {
        let result = combine_series(z.clone(), ImpedanceResult::Short);
        assert_impedance_eq(result, z, EPSILON_STRICT);
    }

#[test]
fn prop_series_identity_right(z in impedance_strategy()) {
        let result = combine_series(ImpedanceResult::Short, z.clone());
        assert_impedance_eq(result, z, EPSILON_STRICT);
    }

#[test]
fn prop_series_annihilator_left(z in impedance_strategy()) {
        let result = combine_series(ImpedanceResult::Open, z);
        prop_assert!(matches!(result, ImpedanceResult::Open));
    }

#[test]
fn prop_series_annihilator_right(z in impedance_strategy()) {
        let result = combine_series(z, ImpedanceResult::Open);
        prop_assert!(matches!(result, ImpedanceResult::Open));
    }

#[test]
fn prop_parallel_identity_left(z in impedance_strategy()) {
        let result = combine_parallel(z.clone(), ImpedanceResult::Open);
        assert_impedance_eq(result, z, EPSILON_STRICT);
    }

#[test]
fn prop_parallel_identity_right(z in impedance_strategy()) {
        let result = combine_parallel(ImpedanceResult::Open, z.clone());
        assert_impedance_eq(result, z, EPSILON_STRICT);
    }

#[test]
fn prop_parallel_annihilator_left(z in impedance_strategy()) {
        let result = combine_parallel(ImpedanceResult::Short, z);
        prop_assert!(matches!(result, ImpedanceResult::Short));
    }

#[test]
fn prop_parallel_annihilator_right(z in impedance_strategy()) {
        let result = combine_parallel(z, ImpedanceResult::Short);
        prop_assert!(matches!(result, ImpedanceResult::Short));
    }
    
#[test]
fn prop_series_many_open_dominates(
        mut impedances in prop::collection::vec(impedance_strategy(), 1..10),
        position in 0..10usize
    ) {
        impedances.insert(position.min(impedances.len()), ImpedanceResult::Open);
        let result = combine_series_many(&impedances);
        prop_assert!(matches!(result, ImpedanceResult::Open));
    }
    
#[test]
fn prop_parallel_many_short_dominates(
        mut impedances in prop::collection::vec(impedance_strategy(), 1..10),
        position in 0..10usize
    ) {
        impedances.insert(position.min(impedances.len()), ImpedanceResult::Short);
        let result = combine_parallel_many(&impedances);
        prop_assert!(matches!(result, ImpedanceResult::Short));
    }

#[test]
fn prop_series_associativity_with_edges(
        a in impedance_strategy(),
        b in impedance_strategy(),
        c in impedance_strategy()
    ) {
        let left = combine_series(combine_series(a.clone(), b.clone()), c.clone());
        let right = combine_series(a, combine_series(b, c));
        assert_impedance_eq(left, right, EPSILON_RELAXED);
    }

#[test]
fn prop_parallel_associativity_with_edges(
        a in impedance_strategy(),
        b in impedance_strategy(),
        c in impedance_strategy()
    ) {
        let left = combine_parallel(combine_parallel(a.clone(), b.clone()), c.clone());
        let right = combine_parallel(a, combine_parallel(b, c));
        assert_impedance_eq(left, right, EPSILON_RELAXED);
    }

#[test]
fn prop_series_commutativity(
    a in impedance_strategy(),
    b in impedance_strategy()
) {
    let left = combine_series(a.clone(), b.clone());
    let right = combine_series(b, a);
    assert_impedance_eq(left, right, EPSILON_STRICT);
}

#[test]
fn prop_parallel_commutativity(
    a in impedance_strategy(),
    b in impedance_strategy()
) {
    let left = combine_parallel(a.clone(), b.clone());
    let right = combine_parallel(b, a);
    assert_impedance_eq(left, right, EPSILON_STRICT);
}
    
}

#[test]
fn parallel_open_consistency() {
    let result = combine_parallel(ImpedanceResult::Open, ImpedanceResult::Open);
    assert!(matches!(result, ImpedanceResult::Open));
}

#[test]
fn series_short_consistency() {
    let result = combine_series(ImpedanceResult::Short, ImpedanceResult::Short);
    assert!(matches!(result, ImpedanceResult::Short));
}

#[test]
fn parallel_many_all_open() {
    let impedances = vec![ImpedanceResult::Open; 5];
    let result = combine_parallel_many(&impedances);
    assert!(matches!(result, ImpedanceResult::Open));
}

#[test]
fn series_many_all_short() {
    let impedances = vec![ImpedanceResult::Short; 5];
    let result = combine_series_many(&impedances);
    assert!(matches!(result, ImpedanceResult::Short));
}

use proptest::prelude::*;
use electro_solve_core::units::*;
use electro_solve_core::component::*;

proptest! {

#[test]
fn prop_resistor_frequency_invariance(
    r_val in 1e-9_f64..1e9_f64,
    omega1 in 1e-6_f64..1e6_f64, 
    omega2 in 1e-6_f64..1e6_f64
) {
    let resistor = ComponentKind::Resistor {
        r: Resistance::known(r_val).unwrap()
    };
    let z1 = resistor.impedance(AngularFrequency::new(omega1).unwrap());
    let z2 = resistor.impedance(AngularFrequency::new(omega2).unwrap());
    match (z1, z2) {
        (ImpedanceResult::Finite(z1_val), ImpedanceResult::Finite(z2_val)) => {
            prop_assert!(z1_val.im.abs() < 1e-10, "Resistor should have no imaginary part");
            prop_assert!(z2_val.im.abs() < 1e-10, "Resistor should have no imaginary part");
            prop_assert!((z1_val.re - z2_val.re).abs() < 1e-10, "Resistor impedance should be frequency-independent");
        },
        _ => prop_assert!(false, "Resistor should always return finite impedance")
    }
}
#[test]
fn prop_inductor_linearity(
    l_val in 1e-9_f64..1e9_f64,
    omega1 in 1e-6_f64..1e6_f64, 
    omega2 in 1e-6_f64..1e6_f64
) {
    let inductor = ComponentKind::Inductor {
        l: Inductance::known(l_val).unwrap()
    };
    let z1 = inductor.impedance(AngularFrequency::new(omega1).unwrap());
    let z2 = inductor.impedance(AngularFrequency::new(omega2).unwrap());
    match (z1, z2) {
        (ImpedanceResult::Finite(z1_val), ImpedanceResult::Finite(z2_val)) => {
            let ratio = omega2 / omega1;
            let mag_ratio = z2_val.norm() / z1_val.norm();
            prop_assert!((mag_ratio - ratio).abs() < 0.01, "Inductor impedance should be linear with frequency");
        },
        _ => prop_assert!(false, "Inductor should return finite impedance for positive frequency")
    }
}
#[test]
fn prop_capacitor_inverse(
    c_val in 1e-9_f64..1e9_f64,
    omega1 in 1e-6_f64..1e6_f64, 
    omega2 in 1e-6_f64..1e6_f64
) {
    let capacitor = ComponentKind::Capacitor {
        c: Capacitance::known(c_val).unwrap()
    };
    let z1 = capacitor.impedance(AngularFrequency::new(omega1).unwrap());
    let z2 = capacitor.impedance(AngularFrequency::new(omega2).unwrap());
    match (z1, z2) {
        (ImpedanceResult::Finite(z1_val), ImpedanceResult::Finite(z2_val)) => {
            let ratio = omega1 / omega2;
            let mag_ratio = z2_val.norm() / z1_val.norm();
            prop_assert!((mag_ratio - ratio).abs() < 0.01, "Capacitor impedance should be inversely proportional to frequency");
        },
        _ => prop_assert!(false, "Capacitor should return finite impedance for positive frequency")
    }
}
#[test]
fn prop_dc_limits(
    r_val in 1e-9_f64..1e9_f64,
    l_val in 1e-9_f64..1e9_f64,
    c_val in 1e-9_f64..1e9_f64
) {
    let resistor = ComponentKind::Resistor { r: Resistance::known(r_val).unwrap() };
    let inductor = ComponentKind::Inductor { l: Inductance::known(l_val).unwrap() };
    let capacitor = ComponentKind::Capacitor { c: Capacitance::known(c_val).unwrap() };
    let dc = AngularFrequency::new(0.0).unwrap();
    // Resistor at DC
    let r_z = resistor.impedance(dc);
    prop_assert!(matches!(r_z, ImpedanceResult::Finite(_)));
    // Inductor at DC (should be short)
    let l_z = inductor.impedance(dc);
    prop_assert!(matches!(l_z, ImpedanceResult::Short));
    // Capacitor at DC (should be open)
    let c_z = capacitor.impedance(dc);
    prop_assert!(matches!(c_z, ImpedanceResult::Open));
}


}

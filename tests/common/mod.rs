use num_complex::Complex64;
use approx::assert_relative_eq;
use electro_solve::units::*;
use electro_solve::component::*;
use electro_solve::graph::*;

pub mod strategies;

/// Epsilon for strict arithmetic comparisons (commutativity, associativity)
/// 
/// Used when comparing results of operations that should be mathematically identical.
/// Example: `z1 + z2` vs `z2 + z1`
pub const EPSILON_STRICT: f64 = 1e-9;

/// Epsilon for relaxed comparisons (associative operations sensitive to order)
/// 
/// Used for operations where floating-point accumulation is significant.
/// Example: `(z1 + z2) + z3` vs `z1 + (z2 + z3)` where order affects precision
pub const EPSILON_RELAXED: f64 = 1e-7;

/// Epsilon for physical property comparisons
/// 
/// Used for component impedance calculations where small errors are acceptable.
/// Example: Verifying `Z = jωL` for inductors
pub const EPSILON_PHYSICAL: f64 = 1e-6;

/// Asserts two complex numbers are approximately equal within a tolerance
/// 
/// # Arguments
/// * `actual` - The computed complex value
/// * `expected` - The expected complex value
/// * `epsilon` - Maximum allowed relative error (default: 1e-6 if both are small)
/// 
/// # Panics
/// Panics if the relative or absolute difference exceeds epsilon
/// 
/// # Example
/// ```rust
/// let z1 = Complex64::new(1.0, 2.0);
/// let z2 = Complex64::new(1.000000001, 2.000000001);
/// assert_complex_eq(z2, z1, EPSILON_STRICT);  // Passes
/// ```
pub fn assert_complex_eq(actual: Complex64, expected: Complex64, epsilon: f64) {
    // Compare real parts
    assert_relative_eq!(actual.re, expected.re, epsilon = epsilon, 
        max_relative = epsilon);
    
    // Compare imaginary parts
    assert_relative_eq!(actual.im, expected.im, epsilon = epsilon, 
        max_relative = epsilon);
}

/// Asserts two impedances are approximately equal, handling Open/Short variants
/// 
/// # Arguments
/// * actual - The computed impedance result
/// * expected - The expected impedance result
/// * epsilon - Tolerance for finite impedance comparisons
/// 
/// # Behavior
/// - If both are Finite, compares complex values with epsilon tolerance
/// - If both are Open, passes (exact match)
/// - If both are Short, passes (exact match)
/// - If variants differ, panics with descriptive message
/// 
/// # Example
/// ```rust
/// let z1 = ImpedanceResult::Finite(Complex64::new(1000.0, 0.0));
/// let z2 = ImpedanceResult::Finite(Complex64::new(1000.0001, 0.0));
/// assert_impedance_eq(z2, z1, EPSILON_PHYSICAL);  // Passes
/// 
/// let z3 = ImpedanceResult::Open;
/// let z4 = ImpedanceResult::Open;
/// assert_impedance_eq(z3, z4, EPSILON_PHYSICAL);  // Passes
/// ```
pub fn assert_impedance_eq(actual: ImpedanceResult, expected: ImpedanceResult, epsilon: f64) {
    match (&actual, &expected) {
        (ImpedanceResult::Finite(actual_z), ImpedanceResult::Finite(expected_z)) => {
            assert_complex_eq(*actual_z, *expected_z, epsilon);
        }
        (ImpedanceResult::Open, ImpedanceResult::Open) => {
            // Open is exact match, no epsilon needed
        }
        (ImpedanceResult::Short, ImpedanceResult::Short) => {
            // Short is exact match, no epsilon needed
        }
        (actual, expected) => {
            panic!(
                "Impedance mismatch: expected {:?}, got {:?}",
                expected, actual
            );
        }
    }
}
/// Checks if an impedance is passive (cannot generate energy)
/// 
/// A passive impedance has a non-negative real part: Re(z) >= 0
/// 
/// # Arguments
/// * z - The complex impedance to check
/// 
/// # Returns
/// true if z.re >= 0, false otherwise
/// 
/// # Physical Interpretation
/// - Passive components (resistors, inductors, capacitors) have Re(z) >= 0
/// - Active components (negative resistance) would have Re(z) < 0 and can generate energy
/// 
/// # Example
/// ```rust
/// assert!(is_passive_impedance(Complex64::new(100.0, 50.0)));  // True
/// assert!(!is_passive_impedance(Complex64::new(-10.0, 0.0)));  // False (active)
/// ```
pub fn is_passive_impedance(z: Complex64) -> bool {
    z.re >= 0.0
}

pub fn is_passive_impedance_result(z: &ImpedanceResult) -> bool {
    match z {
        ImpedanceResult::Finite(z_val) => is_passive_impedance(*z_val),
        ImpedanceResult::Short => true,
        ImpedanceResult::Open => true,
    }
}

use thiserror::Error;
use num_complex::Complex64;

// TODO: Redesign error handling

#[derive(Debug, Error)]
pub enum CircuitError {
    #[error("Invalid angular frequency: {0} (must be ≥ 0 and finite)")]
    InvalidAngularFrequency(f64),
    #[error("Invalid resistance: {0} Ω (must be > 0 and finite)")]
    InvalidResistance(f64),
    #[error("Invalid inductance: {0} H (must be > 0 and finite)")]
    InvalidInductance(f64),
    #[error("Invalid capacitance: {0} F (must be > 0 and finite)")]
    InvalidCapacitance(f64),
    #[error("Invalid impedance: {0} Ω (must be > 0 and finite)")]
    InvalidImpedance(Complex64),
}

#[derive(Debug, Error)]
#[error("Parse error on line {line}: {message}")]
pub struct ParseError { pub line: usize, pub message: String }

impl From<CircuitError> for ParseError {
    fn from(value: CircuitError) -> Self {
        // FIXME: This is temporary. We need proprer line number reporting.
        Self { line: 0, message: format!("{}", value) }
    }
}

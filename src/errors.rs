use thiserror::Error;

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
    #[error("Unknown component type: {0}")]
    UnknownComponentType(String),
    #[error("Invalid SI suffix: {0}")]
    InvalidSuffix(String),
}

#[derive(Debug, Error)]
#[error("Parse error on line {line}: {message}")]
pub struct ParseError { line: usize, message: String }

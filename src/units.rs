use num_complex::Complex64;
use crate::errors::CircuitError;

#[derive(Debug, Clone, PartialEq, Hash)]
pub enum Value<T> {
    Known(T),
    Unknown(String),
}

impl<T> Value<T> {
    pub fn new(value: T) -> Self {
        Self::Known(value)
    }

    pub fn unknown(name: String) -> Self {
        Self::Unknown(name)
    }

    pub fn is_known(&self) -> bool {
        matches!(self, Self::Known(_))
    }

    pub fn is_unknown(&self) -> bool {
        matches!(self, Self::Unknown(_))
    }

    pub fn unwrap_known(&self) -> &T {
        match self {
            Self::Known(value) => value,
            Self::Unknown(_) => panic!("unwrap_known called on unknown value"),
        }
    }
}


#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AngularFrequency(f64);

impl AngularFrequency {
    pub fn new(omega: f64) -> Result<Self, CircuitError> {
        match omega >= 0.0 && omega.is_finite() {
            true => Ok(Self(omega)),
            false => Err(CircuitError::InvalidAngularFrequency(omega)),
        }
    }

    pub fn hz(freq: f64) -> Self {
        Self(2.0 * std::f64::consts::PI * freq)
    }
}

impl From<AngularFrequency> for f64 {
    fn from(value: AngularFrequency) -> f64 {
        value.0
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Resistance(pub Value<f64>);

impl Resistance {
    pub fn known(r: f64) -> Result<Self, CircuitError> {
        match r > 0.0 && r.is_finite() {
            true => Ok(Self(Value::Known(r))),
            false => Err(CircuitError::InvalidResistance(r)),
        }
    }

    pub fn unknown(name: String) -> Self {
        Self(Value::Unknown(name))
    }
    
    pub fn is_known(&self) -> bool {
        self.0.is_known()
    }
    
    pub fn is_unknown(&self) -> bool {
        self.0.is_unknown()
    }
}

impl From<Resistance> for Option<f64> {
    fn from(value: Resistance) -> Self {
        match value.0 {
            Value::Known(r) => Some(r),
            Value::Unknown(_) => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Inductance(pub Value<f64>);

impl Inductance {
    pub fn known(l: f64) -> Result<Self, CircuitError> { 
        match l > 0.0 && l.is_finite() {
            true => Ok(Self(Value::Known(l))),
            false => Err(CircuitError::InvalidInductance(l)),
        }
    }

    pub fn unknown(name: String) -> Self {
        Self(Value::Unknown(name))
    }
    
    pub fn is_known(&self) -> bool {
        self.0.is_known()
    }
    
    pub fn is_unknown(&self) -> bool {
        self.0.is_unknown()
    }
}

impl From<Inductance> for Option<f64> {
    fn from(value: Inductance) -> Self {
        match value.0 {
            Value::Known(l) => Some(l),
            Value::Unknown(_) => None,
        }
    }
}


#[derive(Debug, Clone, PartialEq)]
pub struct Capacitance(pub Value<f64>);

impl Capacitance {
    pub fn known(c: f64) -> Result<Self, CircuitError> {  
        match c > 0.0 && c.is_finite() {
            true => Ok(Self(Value::Known(c))),
            false => Err(CircuitError::InvalidCapacitance(c)),
        }
    }

    pub fn unknown(name: String) -> Self {
        Self(Value::Unknown(name))
    }
    
    pub fn is_known(&self) -> bool {
        self.0.is_known()
    }
    
    pub fn is_unknown(&self) -> bool {
        self.0.is_unknown()
    }
}

impl From<Capacitance> for Option<f64> {
    fn from(value: Capacitance) -> Self {
        match value.0 {
            Value::Known(c) => Some(c),
            Value::Unknown(_) => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Voltage(pub Complex64);

impl Voltage {
    pub fn dc(volts: f64) -> Self {
        Self(Complex64::new(volts, 0.0))
    }
    
    pub fn ac_phasor(magnitude: f64, phase_degrees: f64) -> Self {
        let phase_rad = phase_degrees.to_radians();
        let real = magnitude * phase_rad.cos();
        let imag = magnitude * phase_rad.sin();
        Self(Complex64::new(real, imag))
    }
}

impl Into<Complex64> for Voltage {
    fn into(self) -> Complex64 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Current(pub Complex64);

impl Current {
    pub fn dc(amps: f64) -> Self {
        Self(Complex64::new(amps, 0.0))
    }
    
    pub fn ac_phasor(magnitude: f64, phase_degrees: f64) -> Self {
        let phase_rad = phase_degrees.to_radians();
        let real = magnitude * phase_rad.cos();
        let imag = magnitude * phase_rad.sin();
        Self(Complex64::new(real, imag))
    }
}

impl Into<Complex64> for Current {
    fn into(self) -> Complex64 {
        self.0
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ImpedanceResult {
    Finite(Complex64),
    Open,
    Short,
}
impl ImpedanceResult {
    pub fn is_finite(&self) -> bool { matches!(self, Self::Finite(..)) }
    pub fn is_open(&self) -> bool { matches!(self, Self::Open) }
    pub fn is_short(&self) -> bool { matches!(self, Self::Short) }
}


pub fn combine_series(z1: ImpedanceResult, z2: ImpedanceResult) -> ImpedanceResult {
    match (&z1, &z2) {
        (ImpedanceResult::Open, _) | (_, ImpedanceResult::Open) => {
            ImpedanceResult::Open
        }
        
        (ImpedanceResult::Finite(z1_val), ImpedanceResult::Short)  => { 
            ImpedanceResult::Finite(*z1_val)
        }
        
        (ImpedanceResult::Short, ImpedanceResult::Finite(z2_val))  => { 
            ImpedanceResult::Finite(*z2_val)
        }

        (ImpedanceResult::Short, ImpedanceResult::Short) => {
            ImpedanceResult::Short
        }
        
        (ImpedanceResult::Finite(z1_val), ImpedanceResult::Finite(z2_val)) => {
            ImpedanceResult::Finite(z1_val + z2_val)
        }
    }
}

pub fn combine_parallel(z1: ImpedanceResult, z2: ImpedanceResult) -> ImpedanceResult {
    match (&z1, &z2) {
        (ImpedanceResult::Short, _) | (_, ImpedanceResult::Short) => {
            ImpedanceResult::Short
        }
        
        (ImpedanceResult::Finite(z1_val), ImpedanceResult::Open) => {
            ImpedanceResult::Finite(*z1_val)
        }
        (ImpedanceResult::Open, ImpedanceResult::Finite(z2_val)) => {
            ImpedanceResult::Finite(*z2_val)
        }
        (ImpedanceResult::Open, ImpedanceResult::Open) => {
            ImpedanceResult::Open
        }
        
        (ImpedanceResult::Finite(z1_val), ImpedanceResult::Finite(z2_val)) => {
            let admittance_sum = 1.0 / z1_val + 1.0 / z2_val;
            ImpedanceResult::Finite(1.0 / admittance_sum)
        }
    }
}

pub fn combine_parallel_many(impedances: &[ImpedanceResult]) -> ImpedanceResult {
    // Rule 1: Any Short dominates parallel
    if impedances.iter().any(|z| z.is_short()) {
        return ImpedanceResult::Short;
    }
    
    // Rule 2: Filter out Open (zero admittance)
    let finite_vals: Vec<Complex64> = impedances
        .iter()
        .filter_map(|z| match z {
            ImpedanceResult::Finite(v) => Some(v),
            _ => None,  // Skip Open
        })
        .cloned()
        .collect();
    
    // Rule 3: If all were Open, result is Open
    if finite_vals.is_empty() {
        return ImpedanceResult::Open;
    }
    
    // Rule 4: Calculate parallel impedance
    let admittance_sum: Complex64 = finite_vals
        .iter()
        .map(|z| 1.0 / z)  // Y = 1/Z
        .sum();
    
    ImpedanceResult::Finite(1.0 / admittance_sum)
}

pub fn combine_series_many(impedances: &[ImpedanceResult]) -> ImpedanceResult {
    // Rule 1: Any Open breaks series
    if impedances.iter().any(|z| z.is_open()) {
        return ImpedanceResult::Open;
    }
    
    // Filter out Short (add zero impedance)
    let finite_vals: Vec<Complex64> = impedances
        .iter()
        .filter_map(|z| match z {
            ImpedanceResult::Finite(v) => Some(v),
            ImpedanceResult::Short => None,  // Skip Short
            _ => None,
        })
        .cloned()
        .collect();
    
    // If all were Open (shouldn't happen given first check, but defensive)
    if finite_vals.is_empty() {
        return ImpedanceResult::Open;
    }
    
    // Rule 2: Sum all finite values
    let z_sum: Complex64 = finite_vals
        .iter()
        .sum();
    
    ImpedanceResult::Finite(z_sum)
}

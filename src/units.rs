use num_complex::Complex64;
use crate::errors::CircuitError;

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct AngularFrequency(f64);

impl AngularFrequency {
    pub fn new(omega: f64) -> Result<Self, CircuitError> {
        match omega >= 0.0 && omega.is_finite() {
            true => Ok(Self(omega)),
            false => Err(CircuitError::InvalidAngularFrequency(omega)),
        }
    }

    pub fn from_hz(freq: f64) -> Self {
        Self(2.0 * std::f64::consts::PI * freq.max(0.0))
    }
}

impl From<AngularFrequency> for f64 {
    fn from(value: AngularFrequency) -> f64 {
        value.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Resistance(f64);

impl Resistance {
    pub fn new(r: f64) -> Result<Self, CircuitError> {
        match r > 0.0 && r.is_finite() {
            true => Ok(Self(r)),
            false => Err(CircuitError::InvalidResistance(r)),
        }
    }
}

impl From<Resistance> for f64 {
    fn from(value: Resistance) -> f64 {
        value.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Inductance(f64);

impl Inductance {
    pub fn new(l: f64) -> Result<Self, CircuitError> {
        match l > 0.0 && l.is_finite() {
            true => Ok(Self(l)),
            false => Err(CircuitError::InvalidInductance(l)),
        }
    }
}

impl From<Inductance> for f64 {
    fn from(value: Inductance) -> f64 {
        value.0
    }
}
 

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Capacitance(f64);

impl Capacitance {
    pub fn new(c: f64) -> Result<Self, CircuitError> {
        match c > 0.0 && c.is_finite() {
            true => Ok(Self(c)),
            false => Err(CircuitError::InvalidCapacitance(c)),
        }
    }
}

impl From<Capacitance> for f64 {
    fn from(value: Capacitance) -> f64 {
        value.0
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

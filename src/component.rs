use num_complex::Complex64;
use crate::units::{Resistance, Inductance, Capacitance, Voltage, Current, AngularFrequency};
use crate::errors::CircuitError;

#[derive(Debug, Clone)]
pub enum ComponentKind {
    Resistor {r: Resistance},
    Inductor  {l: Inductance},
    Capacitor {c: Capacitance},
    Impedance {z: Complex64},
    VoltageSource {v: Voltage},
    CurrentSource {i: Current},
}

impl ComponentKind {
    pub fn impedance(&self, omega: AngularFrequency) -> Complex64 {
        match self {
            Self::Resistor {r} => Complex64::new(f64::from(*r), 0.0),
            Self::Inductor {l} => Complex64::new(0.0, f64::from(omega) * f64::from(*l)),
            Self::Capacitor {c} => {
                let omega_val: f64 = f64::from(omega);
                if omega_val < 1e-12 {
                    Complex64::new(1e15, 0.0)
                } else {
                    Complex64::new(0.0, -1.0 / (omega_val * f64::from(*c)))
                }
            },
            Self::Impedance {z} => *z,
            Self::VoltageSource {..} | Self::CurrentSource {..} => Complex64::new(0.0, 0.0)
        }
    }

    pub fn is_passive(&self) -> bool {
        matches!(self, Self::Resistor {..} | Self::Inductor {..} | Self::Capacitor {..} | Self::Impedance {..})
    }

    pub fn is_source(&self) -> bool {
        matches!(self, Self::VoltageSource {..} | Self::CurrentSource {..})
    }

}

pub fn impedance_to_kind(z: Complex64) -> Result<ComponentKind, CircuitError> {
    const EPSILON: f64 = 1e-12;
    if z.im.abs() < EPSILON {
        Resistance::new(z.re).map(|r| ComponentKind::Resistor { r })
    } else {
        Ok(ComponentKind::Impedance { z })
    }
}

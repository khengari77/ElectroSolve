use num_complex::Complex64;
use crate::units::*;
use crate::errors::CircuitError;

#[derive(Debug, Clone, PartialEq)]
pub enum ComponentKind {
    Resistor {r: Resistance},
    Inductor  {l: Inductance},
    Capacitor {c: Capacitance},
    Impedance {z: ImpedanceResult},
    VoltageSource {v: Voltage},
    CurrentSource {i: Current},
}

impl ComponentKind {
    pub fn impedance(&self, omega: AngularFrequency) -> ImpedanceResult {
        match self {
            Self::Resistor {r} => {
                match Option::<f64>::from(r.clone()) {
                    Some(r_val) => ImpedanceResult::new_finite(Complex64::new(r_val, 0.0)),
                    None => ImpedanceResult::Open,
                }
            }
            Self::Inductor {l} => {
                match Option::<f64>::from(l.clone()) {
                    Some(l_val) => ImpedanceResult::new_finite(Complex64::new(0.0, f64::from(omega) * l_val)),
                    None => ImpedanceResult::Open,
                }
            }
            Self::Capacitor {c} => {
                match Option::<f64>::from(c.clone()) {
                    Some(c_val) => {
                        let omega_val: f64 = f64::from(omega);
                        if omega_val < 1e-12 {
                            ImpedanceResult::Open
                        } else {
                            ImpedanceResult::new_finite(Complex64::new(0.0, -1.0 / (omega_val * c_val)))
                        }
                    },
                    None => ImpedanceResult::Open,
                }
            },
            Self::Impedance {z} => z.clone(),
            Self::VoltageSource {..} | Self::CurrentSource {..} => ImpedanceResult::Short
        }
    }

    pub fn is_passive(&self) -> bool {
        match self {
            Self::Resistor {..} | Self::Inductor {..} | Self::Capacitor {..} => true,
            Self::Impedance {z} => z.is_finite(),
            _ => false,
        }
    }

    pub fn is_source(&self) -> bool {
        matches!(self, Self::VoltageSource {..} | Self::CurrentSource {..})
    }

}

pub fn impedance_to_kind(z: ImpedanceResult) -> Result<ComponentKind, CircuitError> {
    const EPSILON: f64 = 1e-12;
    match z {
        ImpedanceResult::Finite(z_val) => {
            if z_val.im.abs() < EPSILON {
                Resistance::known(z_val.re).map(|r| ComponentKind::Resistor { r })
            } else {
                Ok(ComponentKind::Impedance { z: ImpedanceResult::new_finite(z_val) })
            }
        }
        ImpedanceResult::Open => Ok(ComponentKind::Impedance { z: ImpedanceResult::Open }),
        ImpedanceResult::Short => Ok(ComponentKind::Impedance { z: ImpedanceResult::Short }),
    }
}

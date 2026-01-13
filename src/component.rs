use num_complex::Complex64;
use crate::units::{Resistance, Inductance, Capacitance, Voltage, Current, AngularFrequency};

pub enum ComponentKind {
    Resistor {r: Resistance},
    Inductor  {l: Inductance},
    Capacitor {c: Capacitance},
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
            Self::VoltageSource {..} | Self::CurrentSource {..} => Complex64::new(0.0, 0.0)
        }
    }

    pub fn is_passive(&self) -> bool {
        matches!(self, Self::Resistor {..} | Self::Inductor {..} | Self::Capacitor {..})
    }

    pub fn is_source(&self) -> bool {
        matches!(self, Self::VoltageSource {..} | Self::CurrentSource {..})
    }
}


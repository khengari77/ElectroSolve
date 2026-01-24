use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub circuit_name: Option<String>,
    pub ground: Option<String>,
    pub analysis: Option<Analysis>,
    pub elements: Vec<Element>,
    pub constraints: Vec<Constraint>,
    pub solve: Vec<SolveTarget>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Analysis {
    AC { frequency_hz: f64, ac_ref: AcRef },
    DC,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AcRef {
    Rms,
    Peak,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ElemKind {
    Resistor,
    Inductor,
    Capacitor,
    VoltageSource,
    CurrentSource,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Element {
    pub kind: ElemKind,
    pub id: String,
    pub nodes: (String, String),
    pub params: ElementParams,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ElementParams {
    Passive { value: ValueExpr },
    Vac { mag: ValueExpr, phase_deg: f64 },
    Vdc { value: ValueExpr }, 
    Idc { value: ValueExpr },
}

#[derive(Debug, Clone, PartialEq)]
pub enum ValueExpr {
    Known(Quantity),
    Unknown(Symbol),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Symbol(pub String);

#[derive(Debug, Clone, PartialEq)]
pub struct Quantity {
    pub value_si: f64,
    pub unit: Unit,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Unit {
    Ohm,
    Farad,
    Henry,
    Volt,
    Amp,
    Hz,
    Deg,
    Dimensionless,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Constraint {
    pub lhs: Expr,
    pub op: CmpOp,
    pub rhs: Expr
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Sym(Symbol),
    Lit(Quantity),
    V(VoltageReference),
    I(String),
    P(String),
    Zeq(String, String),
    Abs(Box<Expr>),
    AngleDeg(Box<Expr>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CmpOp {
    Eq, Ne, Lt, Le, Gt, Ge,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SolveTarget {
    Expr(Expr),
    Sym(Symbol),
}

#[derive(Debug, Clone, PartialEq)]
pub enum VoltageReference {
    /// Voltage relative to ground (requires ground to be declared)
    GroundRelative(String),
    /// Differential voltage between two nodes (always valid)
    Differential(String, String),
    /// Explicit reference to another node
    NodeRelative(String, String),
}

impl Program {
    pub fn new() -> Self {
        Self {
            circuit_name: None,
            ground: None,
            analysis: None,
            elements: Vec::new(),
            constraints: Vec::new(),
            solve: Vec::new(),
        }
    }

    pub fn has_ground(&self) -> bool {
        self.ground.is_some()
    }

    pub fn requires_ground(&self) -> bool {
        // Check if any expressions require ground
        self.constraints.iter().any(|c| requires_ground_expr(&c.lhs) || requires_ground_expr(&c.rhs)) ||
        self.solve.iter().any(|s| requires_ground_solve_target(s))
    }
}

fn requires_ground_expr(expr: &Expr) -> bool {
    match expr {
        Expr::V(VoltageReference::GroundRelative(_)) => true,
        Expr::Abs(e) | Expr::AngleDeg(e) => requires_ground_expr(e),
        _ => false,
    }
}

fn requires_ground_solve_target(target: &SolveTarget) -> bool {
    match target {
        SolveTarget::Expr(expr) => requires_ground_expr(expr),
        SolveTarget::Sym(_) => false,
    }
}


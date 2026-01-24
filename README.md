# ElectroSolve

ElectroSolve is a Rust project aimed at becoming a “student helper” for circuit analysis: a tool that can take a circuit description and guide you to the answer step-by-step, similar to how Photomath explains math problems.

The long-term goal is not only to compute final results, but to explain *how* they were obtained using standard circuit laws and reduction rules.

This repository is organized as a Rust workspace with separate crates for the core engine, a DSL (circuit description language), and a CLI interface.

---

## Project Goals

ElectroSolve is designed to help students solve circuit problems by supporting:

* Equivalent impedance calculation for AC circuits using complex impedance
* Series/parallel circuit reduction with recorded reduction steps
* Solving for unknown values:

  * Unknown impedances (R, L, C, or general complex impedances)
  * Currents and voltages (phasors for AC, scalars for DC)
  * Power calculations (real, reactive, apparent)
* Step-by-step explanations of each transformation and law used
* A clean input format (DSL) that can later be used by:

  * a CLI tool
  * a future GUI app
  * a web frontend

ElectroSolve is being built with correctness in mind. The core crate already includes extensive property-based testing to validate algebraic and physical invariants.

---

## Current Status

This project is under active development.

### Implemented (Core)

* Circuit graph model (nodes + components)
* Component impedance evaluation at a given angular frequency
* Impedance arithmetic:

  * series combination
  * parallel combination
  * Open/Short edge cases
* Automated reduction loop:

  * parallel reduction
  * series reduction
* Reduction step tracking (`ReductionStep`)
* Strong test coverage using `proptest` and `approx`

### In Progress / Planned

* DSL parser + AST + lowering to `CircuitGraph`
* CLI “solve” command fully wired to DSL input
* Delta–Wye reduction
* Full circuit solving (not just reduction):

  * nodal analysis / MNA for voltages and currents
  * support for independent sources (voltage/current sources)
* Solving unknowns (symbolic or numeric)
* Power reporting and explanation output

---

## Workspace Layout

The workspace is split into three crates:

### `electro_solve_core`

The engine that represents circuits and performs reduction.

Main modules:

* `units`: physical quantities and impedance math
* `component`: component types (R/L/C, impedance, sources)
* `graph`: circuit graph representation
* `reduce`: reduction algorithms and step tracking

### `electro_solve_dsl`

A circuit description language that will allow writing circuits in a simple, readable text format.

Planned pipeline:

1. Parse input into an AST
2. Lower AST into a `CircuitGraph`
3. Solve using the core engine

### `electro_solve_cli`

A command-line interface to run ElectroSolve from the terminal.

---

## How ElectroSolve Thinks About Circuits

ElectroSolve models a circuit as a graph:

* **Nodes** represent connection points (netlist nodes)
* **Components** connect two nodes and have a type:

  * resistor, inductor, capacitor
  * generic impedance
  * (planned) sources and dependent elements

For AC analysis, each component is converted into a complex impedance:

* Resistor:
  ( Z_R = R )

* Inductor:
  ( Z_L = j\omega L )

* Capacitor:
  ( Z_C = \frac{1}{j\omega C} )

The reducer then repeatedly searches for simplifications:

* **Parallel groups**: multiple passive components across the same node pair
* **Series pairs**: two passive components connected through a node with degree 2

Each simplification is recorded as a `ReductionStep`, so the system can later produce a human-readable explanation.

---

## Installation

You need Rust installed (stable toolchain recommended).

Build everything:

```bash
cargo build
```

Run tests:

```bash
cargo test
```

---

## CLI Usage (Early Prototype)

The CLI currently supports a placeholder command:

```bash
cargo run -p electro_solve_cli -- solve <file> <frequency_hz>
```

Example:

```bash
cargo run -p electro_solve_cli -- solve examples/circuit.es 1000
```

At the moment, the CLI reads the file and parses the frequency, but circuit parsing is not implemented yet. This will be connected once the DSL crate is completed.

---

## Example Circuit Input (Planned DSL)

The DSL is still under development, but the intended direction is a simple netlist-like syntax.

Example idea (not final):

```txt
ground gnd

R1 n1 n2 100
C1 n2 gnd 10u
L1 n1 gnd 2m

solve impedance n1 gnd @ 1kHz
```

Planned output (example):

* Equivalent impedance between `n1` and `gnd`
* Reduction steps (series/parallel transforms)
* Optional explanation in text form

---

## Roadmap

### Phase 1: Circuit Reduction (Passive Networks)

* Finish DSL parsing and lowering
* Allow building graphs from text files
* Print reduction steps and final equivalent impedance

### Phase 2: Full Circuit Solving

* Add Modified Nodal Analysis (MNA)
* Support voltage/current sources correctly
* Solve for node voltages and branch currents

### Phase 3: Unknown Solving

* Support unknown values such as `R = x`
* Solve systems of equations numerically
* Add constraints (known voltages, known currents, measured impedance, etc.)

### Phase 4: Student-Friendly Explanations

* Convert reduction/solve steps into readable explanations
* Provide intermediate results and reasoning
* Make outputs suitable for learning, not just computation

---

## Design Principles

* **Correctness first**: property-based tests validate invariants and physical behavior.
* **Separation of concerns**:

  * `core` is the engine
  * `dsl` is the input layer
  * `cli` is the interface
* **Step tracking**: reductions and solving steps should be explainable, not “black box”.

---

## Contributing

This is a student-focused project. Contributions are welcome, especially for:

* DSL grammar design
* Reduction edge cases (especially complex impedance stability)
* MNA implementation and validation
* Explanation formatting and UX

If you plan to contribute, keep changes small and add tests where possible.

---

## License

MIT License. See [LICENSE](./LICENSE).


# ElectroSolve ⚡

**ElectroSolve** is a Python-based electric circuit solver designed for Electrical Engineering students, hobbyists, and professionals. It aims to calculate circuit variables (node voltages, component currents, and voltages) for DC and (eventually) AC circuits. A key feature is its ability to provide a step-by-step symbolic derivation of the solution, helping users understand the underlying principles of circuit analysis.

*This project, up to its current state (Phase 2, Step 2 completion), has been developed in collaboration with Google's Gemini 1.5 Pro large language model.*

## ✨ Features

*   **DC Circuit Analysis:**
    *   Solves circuits with resistors, DC voltage sources, and DC current sources.
    *   Calculates node voltages using Nodal Analysis.
    *   Determines individual component voltages and currents.
*   **Symbolic Step-by-Step Solution (for DC):**
    *   Generates symbolic KCL (Kirchhoff's Current Law) equations for circuit nodes.
    *   Shows explicit voltage definitions from grounded voltage sources.
    *   Displays the substitution of numerical component values into these symbolic equations.
    *   Presents the system of equations to be solved.
    *   Solves the system symbolically using SymPy and shows the results.
*   **JSON-Based Circuit Input:** Define circuits easily using a structured JSON format.
*   **Command-Line Interface:** Simple CLI to load and solve circuits.
*   **Extensible Design:** Built with modularity to support future enhancements like AC analysis and more complex components.

## 🚀 Getting Started

### Prerequisites

*   Python 3.10 or higher
*   Poetry (for dependency management and packaging - recommended)

### Installation

1.  **Clone the repository:**
    ```bash
    git clone https://github.com/khengari77/ElectroSolve.git
    cd ElectroSolve
    ```

2.  **Set up a virtual environment and install dependencies:**

    *   **Using Poetry (recommended):**
        If you don't have Poetry, install it first (see [Poetry's official documentation](https://python-poetry.org/docs/#installation)).
        ```bash
        poetry install
        ```
        This command creates a virtual environment (if one isn't active) and installs all dependencies listed in `pyproject.toml`.

    *   **Using pip with a manual virtual environment:**
        ```bash
        python -m venv .venv
        source .venv/bin/activate  # Linux/macOS
        # .venv\Scripts\activate  # Windows
        pip install numpy scipy sympy
        ```

### Running ElectroSolve

Activate the virtual environment if you haven't already:
```bash
poetry shell  # If using Poetry
# Or: source .venv/bin/activate (or equivalent for your OS)
```

Then, use the `main.py` script to solve a circuit defined in a JSON file:

```bash
python main.py path/to/your/circuit.json
```

An example circuit is provided:
```bash
python main.py examples/simple_dc.json
```

#### Command-Line Options:

*   `--no-numeric`: Skip the numerical solving part and only show symbolic analysis.
*   `--no-symbolic-solve`: Perform symbolic formulation but skip the substitution and solving steps.
*   `--no-symbolic-formulation`: Skip all symbolic analysis steps (formulation and solve).

## 🛠️ Usage

### Circuit Definition (JSON Format)

Circuits are defined in a JSON file. The basic structure is:

```json
{
  "components": [
    {
      "id": "Vs1",
      "type": "voltagesourcedc",
      "value": 9.0,
      "nodes": ["N1", "GND"]
    },
    {
      "id": "R1",
      "type": "resistor",
      "value": 1000.0,
      "nodes": ["N1", "N2"]
    },
    {
      "id": "Is1",
      "type": "currentsourcedc",
      "value": 0.01,
      "nodes": ["N2", "GND"]
    }
    // ... more components
  ],
  "ground_node": "GND"
}
```

**Component Properties:**

*   `id` (string): A unique identifier for the component (e.g., "R1", "Vs1").
*   `type` (string): Type of the component. Currently supported:
    *   `"resistor"`
    *   `"voltagesourcedc"` (Positive terminal is `nodes[0]`, negative is `nodes[1]`)
    *   `"currentsourcedc"` (Current of `value` flows from `nodes[0]` to `nodes[1]` *through the source*)
*   `value` (float): Numerical value of the component (Ohms, Volts, Amps).
*   `nodes` (list of 2 strings): The node IDs the component connects to.

**Global Properties:**

*   `ground_node` (string): The ID of the node to be considered as the 0V reference.

### Output

ElectroSolve will print:

1.  **Circuit Loading Information:** Confirmation of nodes, ground, and components.
2.  **Symbolic Analysis (if not skipped):**
    *   Symbolic representation of components and node voltages.
    *   Initial symbolic KCL equations and explicit voltage definitions.
    *   Substitution of numerical values.
    *   The system of equations after substitution.
    *   Node voltages solved symbolically.
3.  **Numerical DC Solver Results (if not skipped):**
    *   Node voltages.
    *   Voltage drop across and current through each component.

## 🧪 Running Tests

Unit tests are written using Python's `unittest` module. To run tests:

```bash
# Make sure you are in the project root and the virtual environment is active
python -m unittest discover -s tests -p "test_*.py"
```
Or if using Poetry:
```bash
poetry run python -m unittest discover -s tests -p "test_*.py"
```

## 🗺️ Project Roadmap:

ElectroSolve is being developed in phases:

*   **Phase 0: Setup and Foundations** (✅ Completed)
*   **Phase 1: Core DC Resistive Circuit Solver (Numerical)** (✅ Completed)
*   **Phase 2: Basic Step-by-Step Output & Symbolic Foundation** (In Progress)
    *   Symbolic KCL Equation Formulation (✅ Completed)
    *   Symbolic Substitution and Solving (✅ Completed)
    *   Refine step-by-step output detail.
*   **Phase 3: Expanding Capabilities**
    *   Full Modified Nodal Analysis (MNA) for floating voltage sources.
    *   AC Steady-State Analysis (complex numbers, impedances for R, L, C).
    *   Enhanced error handling and circuit validation.
    *   Advanced "step-by-step" (series/parallel simplification, source transformation).
*   **Phase 4: User Interface & Packaging**
    *   Improved CLI.
    *   Potential GUI (PyQt, Kivy, or Web-based).
    *   Packaging for distribution (PyInstaller, etc.).
*   **Phase 5: Mobile App (Long-Term Goal)**

## 🤝 Contributing

Contributions are welcome! If you'd like to contribute, please:

1.  Fork the repository.
2.  Create a new branch for your feature or bug fix.
3.  Write tests for your changes.
4.  Make your changes.
5.  Ensure all tests pass.
6.  Submit a pull request with a clear description of your changes.

Please feel free to open an issue if you find a bug or have a feature request.

## 📜 License

This project is licensed under the MIT License - see the `LICENSE` file for details.

---

*This README was generated based on the project's progress and plan, with significant portions of the codebase developed in collaboration with Google's Gemini 2.5 Pro.*

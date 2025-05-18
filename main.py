import argparse
import json
from electrosolve.parser import load_circuit_from_json
from electrosolve.solver_dc import solve_dc_circuit
from electrosolve.circuit import Circuit, CurrentSourceDC, Resistor, VoltageSourceDC, Component

def main():
    parser = argparse.ArgumentParser(description="ElectroSolve: A Python Circuit Solver")
    parser.add_argument("json_file", help="Path to the JSON file defining the circuit.")
    # parser.add_argument("--output-format", choices=["text", "json"], default="text", help="Format for the output.")
    # parser.add_argument("--verbose", action="store_true", help="Enable verbose output.")

    args = parser.parse_args()

    print(f"Attempting to load circuit from: {args.json_file}")
    try:
        circuit = load_circuit_from_json(args.json_file)
        print("Circuit loaded successfully.")
        print(f"  Nodes: {circuit.nodes}")
        print(f"  Ground Node: {circuit.ground_node}")
        print(f"  Node Map: {circuit.node_map}")
        print(f"  Components: {len(circuit.components)}")
        for comp in circuit.components:
            print(f"    {comp}")


    except FileNotFoundError as e:
        print(f"Error: {e}")
        return
    except ValueError as e: # Covers JSON errors and circuit validation errors from parser
        print(f"Error loading or validating circuit: {e}")
        return
    except Exception as e:
        print(f"An unexpected error occurred during circuit loading: {e}")
        return

    print("\nAttempting to solve the DC circuit...")
    try:
        solve_dc_circuit(circuit)
        print("Circuit solved successfully.")
    except NotImplementedError as e:
        print(f"Solver Error: {e}")
        print("This version of the solver might not support certain component configurations (e.g., floating voltage sources).")
        return
    except RuntimeError as e: # Covers LinAlgError from solver
        print(f"Solver Error: {e}")
        return
    except Exception as e:
        print(f"An unexpected error occurred during solving: {e}")
        return

    print("\n--- Results ---")
    print("Node Voltages:")
    if not circuit.solved_node_voltages:
        print("  No node voltages calculated (e.g., empty or trivial circuit).")
    else:
        # Sort by node name for consistent output, ground first if present
        sorted_nodes = sorted(
            circuit.solved_node_voltages.keys(),
            key=lambda x: (x != circuit.ground_node, x) # Puts ground first, then alphanumeric
        )
        for node_name in sorted_nodes:
            voltage = circuit.solved_node_voltages[node_name]
            print(f"  V({node_name}): {voltage:.4f} V")

    print("\nComponent Details:")
    if not circuit.components:
        print("  No components in the circuit.")
    else:
        for comp in circuit.components:
            # Determine a readable representation for current
            current_str = "N/A (not calculated)"
            if comp.current is not None:
                if isinstance(comp.current, (int, float)) and abs(comp.current) < 1e-9: # Treat very small as 0
                    current_val = 0.0
                else:
                    current_val = comp.current

                # Unit prefixing for current (Can be moved to a util function later)
                if abs(current_val) >= 1:
                    unit = "A"
                    display_current = current_val
                elif abs(current_val) >= 1e-3:
                    unit = "mA"
                    display_current = current_val * 1e3
                elif abs(current_val) >= 1e-6:
                    unit = "µA"
                    display_current = current_val * 1e6
                elif abs(current_val) >= 1e-9:
                    unit = "nA"
                    display_current = current_val * 1e9
                else:
                    unit = "A"
                    display_current = current_val
                current_str = f"{display_current:.3f} {unit}"

            voltage_str = "N/A"
            if comp.voltage is not None:
                 voltage_str = f"{comp.voltage:.4f} V"


            print(f"  {comp.id} ({comp.__class__.__name__}): Value={comp.value}")
            print(f"    Nodes: {comp.nodes}")
            print(f"    Voltage Drop ({comp.nodes[0]} to {comp.nodes[1]}): {voltage_str}")
            print(f"    Current (flowing {comp.nodes[0]} to {comp.nodes[1]} if positive for R): {current_str}")
            if isinstance(comp, CurrentSourceDC):
                 print(f"      (Note: Positive current for CurrentSourceDC means current flows {comp.nodes[0]} -> {comp.nodes[1]} through source)")
            elif isinstance(comp, Resistor) and comp.current is not None:
                # For resistors, positive current means flow from node with higher potential to lower.
                # Our comp.voltage = v1 - v2, comp.current = comp.voltage / R.
                # So if comp.current is positive, current flows from comp.nodes[0] to comp.nodes[1].
                pass


if __name__ == "__main__":
    main()

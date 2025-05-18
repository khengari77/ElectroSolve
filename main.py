# main.py
import argparse
import json
from electrosolve.parser import load_circuit_from_json
from electrosolve.solver_dc import solve_dc_circuit # For numerical solution
# --- Added for Phase 2 ---
import sympy
from electrosolve.solver_dc import formulate_symbolic_dc_equations
# --- End Phase 2 Additions ---
from electrosolve.circuit import Circuit, CurrentSourceDC, Resistor, VoltageSourceDC, Component

def main():
    parser = argparse.ArgumentParser(description="ElectroSolve: A Python Circuit Solver")
    parser.add_argument("json_file", help="Path to the JSON file defining the circuit.")
    parser.add_argument("--no-numeric", action="store_true", help="Skip numerical solving and only show symbolic equations.")
    parser.add_argument("--no-symbolic", action="store_true", help="Skip symbolic formulation.")


    args = parser.parse_args()

    print(f"Attempting to load circuit from: {args.json_file}")
    try:
        circuit = load_circuit_from_json(args.json_file)
        print("Circuit loaded successfully.")
        print(f"  Nodes: {circuit.nodes}")
        print(f"  Ground Node: {circuit.ground_node}")
        # Ensure node_map is built if not already (parser calls it, but defensive)
        if not circuit.node_map and (circuit.nodes - {circuit.ground_node}):
            circuit.build_node_map()
        print(f"  Node Map (for matrix): {circuit.node_map}")
        print(f"  Components: {len(circuit.components)}")
        # for comp in circuit.components: # Can be verbose
        #     print(f"    {comp}")

    except FileNotFoundError as e:
        print(f"Error: {e}")
        return
    except ValueError as e: 
        print(f"Error loading or validating circuit: {e}")
        return
    except Exception as e:
        print(f"An unexpected error occurred during circuit loading: {e}")
        return

    if not args.no_symbolic:
        print("\n--- Symbolic Equation Formulation ---")
        try:
            kcl_eqs, explicit_defs, base_node_syms, comp_val_syms = formulate_symbolic_dc_equations(circuit)
            
            if not base_node_syms and not comp_val_syms : # No non-ground nodes or components
                 print("No elements for symbolic representation (e.g., empty or ground-only circuit).")
            else:
                print("\nComponent Value Symbols:")
                if not comp_val_syms: print("  None")
                for comp_id, sym in sorted(comp_val_syms.items()): # Sort for consistent output
                    print(f"  {comp_id}: {sym}")

                print("\nBase Node Voltage Symbols (for non-ground nodes):")
                if not base_node_syms: print("  None")
                for node_name, sym in sorted(base_node_syms.items()):
                    print(f"  Node {node_name}: {sym}")
                
                unknown_node_voltage_symbols = {
                    name: sym for name, sym in base_node_syms.items() if name not in explicit_defs
                }
                if unknown_node_voltage_symbols:
                    print("\nConsidered Unknown Node Voltage Symbols (to be solved for):")
                    for name, sym in sorted(unknown_node_voltage_symbols.items()):
                        print(f"  {sym} (for Node {name})")
                else:
                    print("\nNo unknown node voltages to solve for (all non-ground nodes might be defined by sources).")


                if explicit_defs:
                    print("\nExplicit Voltage Definitions (from grounded voltage sources):")
                    for node_name in sorted(explicit_defs.keys()):
                        eq_pretty = sympy.pretty(explicit_defs[node_name], use_unicode=True)
                        print(f"  For Node {node_name}: {eq_pretty}")
                
                if kcl_eqs:
                    print("\nSymbolic KCL Equations (for nodes with unknown voltages):")
                    for node_name in sorted(kcl_eqs.keys()):
                        # eq_simplified = sympy.simplify(kcl_eqs[node_name]) # Simplification can be slow/complex
                        eq_pretty = sympy.pretty(kcl_eqs[node_name], use_unicode=True)
                        print(f"  KCL at Node {node_name}:\n {eq_pretty}")
                elif not explicit_defs and base_node_syms : # Has unknown nodes but no KCL and no explicit defs
                    print("\nNo KCL equations generated for unknown nodes (circuit might be trivial or disconnected).")
                elif not kcl_eqs and explicit_defs and not unknown_node_voltage_symbols:
                     print("\nAll non-ground node voltages are explicitly defined by sources; no further KCL equations needed for solving.")


        except NotImplementedError as e:
            print(f"Symbolic Formulation Error: {e}")
        except ValueError as e:
            print(f"Symbolic Formulation Error: {e}")
        except Exception as e:
            print(f"An unexpected error occurred during symbolic formulation: {e}")
            import traceback
            traceback.print_exc() # For debugging during development

    if not args.no_numeric:
        print("\n--- Numerical DC Solver ---")
        try:
            solve_dc_circuit(circuit)
            print("Circuit solved numerically successfully.")
            
            print("\nNode Voltages (Numerical):")
            if not circuit.solved_node_voltages:
                print("  No node voltages calculated.")
            else:
                sorted_nodes = sorted(
                    circuit.solved_node_voltages.keys(),
                    key=lambda x: (x != circuit.ground_node, x) 
                )
                for node_name in sorted_nodes:
                    voltage = circuit.solved_node_voltages[node_name]
                    print(f"  V({node_name}): {voltage:.4f} V")

            print("\nComponent Details (Numerical):")
            if not circuit.components:
                print("  No components in the circuit.")
            else:
                for comp in circuit.components:
                    current_str = "N/A"
                    if comp.current is not None:
                         current_val = comp.current
                         if abs(current_val) < 1e-12: current_val = 0.0 # Threshold small currents
                         if abs(current_val) >= 1: unit, factor = "A", 1
                         elif abs(current_val) >= 1e-3: unit, factor = "mA", 1e3
                         elif abs(current_val) >= 1e-6: unit, factor = "µA", 1e6
                         else: unit, factor = "nA", 1e9
                         current_str = f"{current_val * factor:.3f} {unit}"
                         if unit == "nA" and abs(current_val * factor) < 0.001 : # for very very small nA
                            current_str = f"{current_val * 1e12:.3f} pA"


                    voltage_str = "N/A"
                    if comp.voltage is not None:
                        voltage_str = f"{comp.voltage:.4f} V"
                    
                    print(f"  {comp.id} ({comp.__class__.__name__}): Value={comp.value}")
                    print(f"    Nodes: {comp.nodes}")
                    print(f"    Voltage Drop ({comp.nodes[0]} to {comp.nodes[1]}): {voltage_str}")
                    print(f"    Current: {current_str}")
                    if isinstance(comp, Resistor) and comp.current is not None:
                        print(f"      (Positive current indicates flow from {comp.nodes[0]} to {comp.nodes[1]})")
                    elif isinstance(comp, CurrentSourceDC):
                         print(f"      (Source current defined as flowing {comp.nodes[0]} to {comp.nodes[1]} through source)")


        except NotImplementedError as e:
            print(f"Numerical Solver Error: {e}")
        except RuntimeError as e: 
            print(f"Numerical Solver Error: {e}")
        except Exception as e:
            print(f"An unexpected error occurred during numerical solving: {e}")
            import traceback
            traceback.print_exc()

if __name__ == "__main__":
    main()

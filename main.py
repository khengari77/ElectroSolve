import argparse
import json
import sympy


from electrosolve.parser import load_circuit_from_json
from electrosolve.solver_dc import (
    solve_dc_circuit, 
    formulate_symbolic_dc_equations,
)
from electrosolve.symbolic_handler import (
    solve_symbolically_after_formulation
)
from electrosolve.circuit import Circuit, CurrentSourceDC, Resistor, VoltageSourceDC, Component # Component was missing


def main():
    parser = argparse.ArgumentParser(description="ElectroSolve: A Python Circuit Solver")
    parser.add_argument("json_file", help="Path to the JSON file defining the circuit.")
    parser.add_argument("--no-numeric", action="store_true", help="Skip numerical solving.")
    parser.add_argument("--no-symbolic-solve", action="store_true", help="Skip symbolic substitution and solving (only shows initial KCLs).")
    parser.add_argument("--no-symbolic-formulation", action="store_true", help="Skip all symbolic steps (formulation and solve).")


    args = parser.parse_args()

    print(f"Attempting to load circuit from: {args.json_file}")
    try:
        circuit = load_circuit_from_json(args.json_file)
        print("Circuit loaded successfully.")
        print(f"  Nodes: {circuit.nodes}")
        print(f"  Ground Node: {circuit.ground_node}")
        if not circuit.node_map and (circuit.nodes - {circuit.ground_node}):
            circuit.build_node_map()
        print(f"  Node Map (for matrix): {circuit.node_map}")
        print(f"  Components: {len(circuit.components)}")

    except FileNotFoundError as e:
        print(f"Error: {e}")
        return
    except ValueError as e: 
        print(f"Error loading or validating circuit: {e}")
        return
    except Exception as e:
        print(f"An unexpected error occurred during circuit loading: {e}")
        return

    # --- Symbolic Section ---
    if not args.no_symbolic_formulation:
        print("\n--- Symbolic Analysis ---")
        kcl_eqs, explicit_defs, base_node_syms, comp_val_syms = {}, {}, {}, {} # Initialize
        symbolic_formulation_ok = False
        try:
            print("1. Formulating Symbolic Equations...")
            kcl_eqs, explicit_defs, base_node_syms, comp_val_syms = formulate_symbolic_dc_equations(circuit)
            symbolic_formulation_ok = True
            
            if not base_node_syms and not comp_val_syms :
                 print("  No elements for symbolic representation (e.g., empty or ground-only circuit).")
            else:
                print("\n  Component Value Symbols:")
                if not comp_val_syms: print("    None")
                for comp_id, sym in sorted(comp_val_syms.items()):
                    print(f"    {comp_id}: {sym}")

                print("\n  Base Node Voltage Symbols (for non-ground nodes):")
                if not base_node_syms: print("    None")
                for node_name, sym in sorted(base_node_syms.items()):
                    print(f"    Node {node_name}: {sym}")
                
                unknown_node_voltage_symbols_initial = {
                    name: sym for name, sym in base_node_syms.items() if name not in explicit_defs
                }
                if unknown_node_voltage_symbols_initial:
                    print("\n  Considered Unknown Node Voltage Symbols (before explicit definitions):")
                    for name, sym in sorted(unknown_node_voltage_symbols_initial.items()):
                        print(f"    {sym} (for Node {name})")
                
                if explicit_defs:
                    print("\n  Initial Explicit Voltage Definitions (from grounded sources):")
                    for node_name in sorted(explicit_defs.keys()):
                        eq_pretty = sympy.pretty(explicit_defs[node_name], use_unicode=True)
                        print(f"    For Node {node_name}: {eq_pretty}")
                
                if kcl_eqs:
                    print("\n  Initial Symbolic KCL Equations (for nodes potentially needing solving):")
                    for node_name in sorted(kcl_eqs.keys()):
                        eq_pretty = sympy.pretty(kcl_eqs[node_name], use_unicode=True)
                        print(f"    KCL at Node {node_name}:\n {eq_pretty}")
                elif not explicit_defs and base_node_syms :
                    print("\n  No KCL equations generated and no explicit definitions for unknown nodes (circuit might be trivial or disconnected).")
                elif not kcl_eqs and explicit_defs and not unknown_node_voltage_symbols_initial:
                     print("\n  All non-ground node voltages seem explicitly defined by sources; no KCL equations needed for solving.")

        except NotImplementedError as e:
            print(f"  Symbolic Formulation Error: {e}")
        except ValueError as e:
            print(f"  Symbolic Formulation Error: {e}")
        except Exception as e:
            print(f"  An unexpected error occurred during symbolic formulation: {e}")
            import traceback
            traceback.print_exc()
        
        # --- Symbolic Substitution and Solve ---
        if symbolic_formulation_ok and not args.no_symbolic_solve:
            print("\n2. Substituting Numerical Values and Solving Symbolically...")
            try:
                # Call the new solver function
                (solved_voltages_symbol_map, 
                 substituted_explicit_defs, 
                 substituted_kcl_eqs,
                 value_sub_dict) = solve_symbolically_after_formulation(
                                        circuit, kcl_eqs, explicit_defs, 
                                        base_node_syms, comp_val_syms
                                    )

                print("\n  Component Values Used for Substitution:")
                if not value_sub_dict: print("    None")
                # Filter to show only component symbols, not substituted node voltages yet
                comp_subs_dict = {s: v for s, v in value_sub_dict.items() if s in comp_val_syms.values()}
                for sym, val in sorted(comp_subs_dict.items(), key=lambda item: str(item[0])):
                    comp_obj = circuit.get_component_by_symbol(sym, comp_val_syms) # Helper needed in Circuit or here
                    unit = ""
                    if comp_obj:
                        if isinstance(comp_obj, Resistor): unit = "Ohm"
                        elif isinstance(comp_obj, VoltageSourceDC): unit = "V"
                        elif isinstance(comp_obj, CurrentSourceDC): unit = "A"
                    print(f"    {sym} = {val} {unit}")
                
                if substituted_explicit_defs:
                    print("\n  Explicit Definitions After Substitution:")
                    for node_name in sorted(substituted_explicit_defs.keys()):
                        eq_pretty = sympy.pretty(substituted_explicit_defs[node_name], use_unicode=True)
                        print(f"    For Node {node_name}: {eq_pretty}")
                
                if substituted_kcl_eqs:
                    print("\n  KCL Equations After Substitution (System to be Solved):")
                    for node_name in sorted(substituted_kcl_eqs.keys()):
                        eq_pretty = sympy.pretty(substituted_kcl_eqs[node_name], use_unicode=True)
                        print(f"    KCL at Node {node_name}: {eq_pretty}")
                elif not substituted_explicit_defs and base_node_syms :
                    print("\n  No KCL equations to solve (circuit might be trivial or disconnected).")
                elif not substituted_kcl_eqs and substituted_explicit_defs:
                     # Check if all base_node_syms are covered by solved_voltages_symbol_map
                    all_defined = all(sym in solved_voltages_symbol_map for sym in base_node_syms.values())
                    if all_defined:
                        print("\n  All non-ground node voltages determined by explicit definitions; no KCL system to solve.")
                    else:
                        print("\n  No KCL equations, but some node voltages remain undefined after explicit definitions. Possible issue.")


                print("\n  Solved Node Voltages (Symbolic Method):")
                if not solved_voltages_symbol_map and base_node_syms:
                    print("    No solution found or no unknown voltages to solve for.")
                elif not base_node_syms:
                    print("    No non-ground nodes to solve for.")
                else:
                    # Add ground node for complete output
                    print(f"    V({circuit.ground_node}) = 0.0000 V (by definition)")
                    
                    # Map solved symbols back to node names for printing
                    solved_node_voltages_by_name = {}
                    for node_name, base_sym in base_node_syms.items():
                        if base_sym in solved_voltages_symbol_map:
                            solved_node_voltages_by_name[node_name] = solved_voltages_symbol_map[base_sym]
                        else:
                            # This node was not solved for (e.g., if system was unsolvable for it)
                            solved_node_voltages_by_name[node_name] = "Not Solved"


                    for node_name in sorted(base_node_syms.keys()): # Iterate to maintain order
                        val = solved_node_voltages_by_name.get(node_name)
                        if isinstance(val, sympy.Number):
                            print(f"    V({node_name}) = {float(val):.4f} V (from symbol {base_node_syms[node_name]})")
                        else:
                             print(f"    V({node_name}) = {val} (symbol {base_node_syms[node_name]})")
                
                # TODO (Optional): Calculate and display component currents/voltages using these solved values
                # This would be similar to the numerical part but using the `solved_voltages_symbol_map`
                # and `base_node_syms` to get numerical V(node) for calculations.

            except NotImplementedError as e:
                print(f"  Symbolic Substitution/Solve Error: {e}")
            except ValueError as e:
                print(f"  Symbolic Substitution/Solve Error: {e}")
            except Exception as e:
                print(f"  An unexpected error occurred during symbolic substitution/solve: {e}")
                import traceback
                traceback.print_exc()
        elif symbolic_formulation_ok and args.no_symbolic_solve:
            print("\n2. Symbolic substitution and solving skipped by user.")


    # --- Numerical Solver Section ---
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

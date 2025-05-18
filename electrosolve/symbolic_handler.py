# electrosolve/symbolic_handler.py
import sympy
from .circuit import Circuit, Resistor, VoltageSourceDC, CurrentSourceDC, Component


def create_node_voltage_symbols(circuit: Circuit) -> dict[str, sympy.Symbol]:
    """
    Creates SymPy symbols for each non-ground node's voltage.
    e.g., 'N1' -> V_N1
    """
    symbols = {}
    # circuit.node_map only contains non-ground nodes
    for node_name in circuit.node_map.keys():
        symbols[node_name] = sympy.symbols(f"V_{sanitize_for_symbol(node_name)}")
    return symbols

def create_component_value_symbols(circuit: Circuit) -> dict[str, sympy.Symbol]:
    """
    Creates SymPy symbols for each component's value.
    e.g., Resistor 'R1' -> R_R1, VoltageSourceDC 'Vs1' -> V_Vs1
    """
    symbols = {}
    for comp in circuit.components:
        comp_id_sanitized = sanitize_for_symbol(comp.id)
        symbol_name_prefix = ""
        if isinstance(comp, Resistor):
            symbol_name_prefix = "R"
        elif isinstance(comp, VoltageSourceDC):
            symbol_name_prefix = "V"
        elif isinstance(comp, CurrentSourceDC):
            symbol_name_prefix = "I"
        else:
            symbol_name_prefix = "Val" # Generic for other types

        symbols[comp.id] = sympy.symbols(f"{symbol_name_prefix}_{comp_id_sanitized}")
    return symbols

# In solver_dc.py (conceptual)

def solve_symbolically_after_formulation(
    circuit: Circuit,
    kcl_equations_for_unknowns: dict[str, sympy.Eq],
    explicit_voltage_definitions: dict[str, sympy.Eq],
    base_node_voltage_symbols: dict[str, sympy.Symbol],
    component_value_symbols: dict[str, sympy.Symbol]
) -> tuple[
    dict[sympy.Symbol, sympy.Number], # Solved node voltage symbols to values
    dict[str, sympy.Eq],              # Substituted explicit definitions
    dict[str, sympy.Eq],              # Substituted KCL equations
    dict[sympy.Symbol, float]         # Symbol to value map used for substitution
    ]:

    # 1. Create value substitution dictionary
    value_sub_dict = {}
    for comp_id, sym in component_value_symbols.items():
        comp = circuit.get_component(comp_id)
        if comp:
            value_sub_dict[sym] = sympy.Float(comp.value) # Use sympy.Float for precision
        else:
            # Should not happen if symbols are generated correctly from circuit components
            print(f"Warning: Component {comp_id} for symbol {sym} not found in circuit.")

    # Store solved node voltages (symbol: value)
    solved_node_voltages_map_sym: dict[sympy.Symbol, sympy.Number] = {}

    # 2. Substitute into explicit definitions
    substituted_explicit_defs: dict[str, sympy.Eq] = {}
    for node_name, eq in explicit_voltage_definitions.items():
        # eq is V_NodeX = V_SourceSym or V_NodeX = -V_SourceSym
        # RHS is the source symbol or its negation.
        rhs_expr = eq.rhs
        substituted_rhs = rhs_expr.subs(value_sub_dict)
        
        node_sym = base_node_voltage_symbols[node_name]
        substituted_explicit_defs[node_name] = sympy.Eq(node_sym, substituted_rhs)
        
        if isinstance(substituted_rhs, sympy.Number):
            solved_node_voltages_map_sym[node_sym] = substituted_rhs
        else:
            # This could happen if a source value was symbolic itself, not planned for now
            print(f"Warning: Explicit definition for {node_name} did not resolve to a number: {substituted_rhs}")

    # Add these solved voltages to the value_sub_dict for further substitutions into KCL
    value_sub_dict_for_kcl = value_sub_dict.copy()
    value_sub_dict_for_kcl.update(solved_node_voltages_map_sym)


    # 3. Substitute into KCL equations
    final_equations_to_solve = []
    unknown_symbols_to_solve_for = []
    substituted_kcl_eqs_output: dict[str, sympy.Eq] = {}

    for node_name, eq in kcl_equations_for_unknowns.items():
        substituted_eq = eq.subs(value_sub_dict_for_kcl)
        final_equations_to_solve.append(substituted_eq)
        substituted_kcl_eqs_output[node_name] = substituted_eq
        
        # Identify remaining unknown symbols in this equation
        # These are the V_Node symbols that weren't in solved_node_voltages_map_sym
        for atom in substituted_eq.free_symbols:
            if atom not in value_sub_dict_for_kcl and atom not in unknown_symbols_to_solve_for:
                if str(atom).startswith("V_"): # Heuristic for node voltage symbols
                    unknown_symbols_to_solve_for.append(atom)
    
    # Ensure we only try to solve for symbols that are actually in base_node_voltage_symbols
    # and not already solved via explicit definitions.
    actual_unknowns = [
        sym for name, sym in base_node_voltage_symbols.items()
        if sym not in solved_node_voltages_map_sym and sym in unknown_symbols_to_solve_for
    ]
    
    # 4. Solve the system
    solution_symbolic = {}
    if final_equations_to_solve and actual_unknowns:
        try:
            solution_symbolic = sympy.solve(final_equations_to_solve, actual_unknowns, dict=True)
            if isinstance(solution_symbolic, list) and solution_symbolic: # sympy often returns a list of dicts
                solution_symbolic = solution_symbolic[0]
            elif not isinstance(solution_symbolic, dict):
                solution_symbolic = {} # Ensure it's a dict
        except Exception as e:
            print(f"SymPy solver error: {e}")
            # This can happen if system is under/over determined after substitution
            # or other sympy issues.
            solution_symbolic = {} 
    
    # Combine explicitly solved and KCL-solved voltages
    final_solved_voltages = solved_node_voltages_map_sym.copy()
    final_solved_voltages.update(solution_symbolic)

    return final_solved_voltages, substituted_explicit_defs, substituted_kcl_eqs_output, value_sub_dict

def sanitize_for_symbol(name: str) -> str:
    """
    Sanitizes a string to be more suitable for use in a SymPy symbol name,
    especially if it might be used to auto-generate variable names later.
    """
    # Replace common problematic characters with underscores
    # Allow alphanumeric and underscore to remain.
    # This is a basic sanitizer; more complex names might need more rules.
    import re
    name = re.sub(r'[^a-zA-Z0-9_]', '_', name)
    # Ensure it doesn't start with a digit if it could be an issue (less so for V_Name symbols)
    # if name and name[0].isdigit():
    #     name = "_" + name # Prepend underscore if starts with digit
    return name

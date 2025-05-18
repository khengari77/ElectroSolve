# electrosolve/symbolic_handler.py
import sympy
from .circuit import Circuit, Resistor, VoltageSourceDC, CurrentSourceDC, Component

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

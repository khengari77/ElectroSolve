import numpy as np
from .circuit import Circuit, Resistor, VoltageSourceDC, CurrentSourceDC
import sympy
from .symbolic_handler import (
    create_node_voltage_symbols,
    create_component_value_symbols,
)


def solve_dc_circuit(circuit: Circuit):
    if circuit.ground_node is None:
        raise ValueError("Circuit must have a ground node set before solving.")
    if not circuit.node_map and circuit.nodes != {circuit.ground_node} and circuit.nodes:
        circuit.build_node_map()

    N = circuit.num_non_ground_nodes

    if N == 0:
        circuit.solution_v = np.array([])
        circuit.solved_node_voltages = {circuit.ground_node: 0.0} if circuit.ground_node else {}
        for comp in circuit.components:
            # Assume components only connect to ground if N=0 (otherwise circuit is ill-defined)
            node1_name, node2_name = comp.nodes[0], comp.nodes[1]
            if node1_name == circuit.ground_node and node2_name == circuit.ground_node:
                comp.voltage = 0.0
                if isinstance(comp, Resistor):
                    comp.current = 0.0 if comp.value != 0 else np.nan
                elif isinstance(comp, CurrentSourceDC):
                    comp.current = comp.value
                    # If Is is across GND-GND, it implies a contradiction unless value is 0.
                    # We assume valid circuits for now.
                elif isinstance(comp, VoltageSourceDC):
                    if not np.isclose(comp.value, 0.0):
                        # This indicates an issue: VSource across ground must be 0V.
                        print(f"Warning: Voltage source {comp.id} ({comp.value}V) connected across ground.")
                    comp.current = None # Cannot determine without more info
            else:
                # This case (N=0 but component connects to a non-ground node name)
                # should be an error from parser or circuit validation ideally.
                 print(f"Warning: Component {comp.id} connects to non-ground nodes but N=0.")
        return

    G = np.zeros((N, N), dtype=float)
    I = np.zeros(N, dtype=float)

    # --- Stamp components assuming all non-ground nodes are initially unknown ---
    for comp in circuit.components:
        node1_name, node2_name = comp.nodes[0], comp.nodes[1]
        idx1 = circuit.node_map.get(node1_name, -1)
        idx2 = circuit.node_map.get(node2_name, -1)

        if isinstance(comp, Resistor):
            conductance = 1.0 / comp.value
            if idx1 != -1:
                G[idx1, idx1] += conductance
            if idx2 != -1:
                G[idx2, idx2] += conductance
            if idx1 != -1 and idx2 != -1:
                G[idx1, idx2] -= conductance
                G[idx2, idx1] -= conductance

        elif isinstance(comp, CurrentSourceDC):
            # Current from nodes[0] to nodes[1] through the source.
            # Current is drawn from nodes[0] (-value) and injected into nodes[1] (+value).
            if idx1 != -1:
                I[idx1] -= comp.value
            if idx2 != -1:
                I[idx2] += comp.value

        elif isinstance(comp, VoltageSourceDC):
            # Voltage sources are handled next by modifying G and I.
            # For now, ensure they are grounded.
            is_node1_ground = (idx1 == -1)
            is_node2_ground = (idx2 == -1)
            if not is_node1_ground and not is_node2_ground:
                raise NotImplementedError(
                    f"Floating voltage source {comp.id} ({node1_name}-{node2_name}) is not supported "
                    "in this simplified DC solver. One terminal must be ground."
                )

    # --- Apply constraints from grounded voltage sources ---
    voltage_source_modifications = [] # To avoid modifying while iterating if needed, though direct is fine here
    for comp in circuit.components:
        if isinstance(comp, VoltageSourceDC):
            node1_name, node2_name = comp.nodes[0], comp.nodes[1] # node1 is positive
            idx1 = circuit.node_map.get(node1_name, -1)
            idx2 = circuit.node_map.get(node2_name, -1)

            if idx1 != -1 and idx2 == -1: # Vs between Node1 and Ground (Node1 = Vs)
                fixed_node_idx, fixed_value = idx1, comp.value
            elif idx1 == -1 and idx2 != -1: # Vs between Ground and Node2 (Node2 = -Vs)
                fixed_node_idx, fixed_value = idx2, -comp.value
            else:
                # This case should be caught by the earlier check if both are non-ground.
                # If both are ground, it's a 0V source, no non-ground node to fix.
                continue

            # Modify G and I for the fixed node
            G[fixed_node_idx, :] = 0.0    # Zero out the entire row
            G[fixed_node_idx, fixed_node_idx] = 1.0 # Set diagonal to 1
            I[fixed_node_idx] = fixed_value       # Set RHS to the fixed voltage

    # --- Solve the system ---
    try:
        # print("G matrix:\n", G) # For debugging
        # print("I vector:\n", I) # For debugging
        solved_voltages_vector = np.linalg.solve(G, I)
        circuit.solution_v = solved_voltages_vector
    except np.linalg.LinAlgError as e:
        raise RuntimeError(f"Failed to solve circuit: Linear algebra error ({e}). "
                           "Check for issues like floating sub-circuits, inconsistent/redundant voltage sources, "
                           "or a current source forming a loop with only other current sources.") from e

    # --- Store node voltages ---
    circuit.solved_node_voltages = {}
    if circuit.ground_node:
        circuit.solved_node_voltages[circuit.ground_node] = 0.0

    for node_name, idx in circuit.node_map.items():
        circuit.solved_node_voltages[node_name] = solved_voltages_vector[idx]

    # --- Calculate and store component voltages and currents ---
    for comp in circuit.components:
        node1_name, node2_name = comp.nodes[0], comp.nodes[1]

        # Ensure nodes exist in solved_node_voltages (should always, due to ground and solved vector)
        v1 = circuit.solved_node_voltages.get(node1_name, 0.0 if node1_name == circuit.ground_node else np.nan)
        v2 = circuit.solved_node_voltages.get(node2_name, 0.0 if node2_name == circuit.ground_node else np.nan)

        if np.isnan(v1) or np.isnan(v2):
            # This might happen if a component references a node not in node_map and not ground
            # which should ideally be caught earlier.
            print(f"Warning: Could not find voltage for nodes of component {comp.id}. Nodes: {comp.nodes}")
            comp.voltage = np.nan
            comp.current = np.nan
            continue

        comp.voltage = v1 - v2 # Voltage across component (V_node1 - V_node2)

        if isinstance(comp, Resistor):
            if comp.value == 0: # Ideal wire / short circuit
                # Voltage should be 0 if well-defined. Current is indeterminate from this formula.
                if not np.isclose(comp.voltage, 0.0):
                    print(f"Warning: Non-zero voltage {comp.voltage}V across zero-ohm resistor {comp.id}.")
                comp.current = np.nan # Current through a short is not found by V/R
            else:
                comp.current = comp.voltage / comp.value
        elif isinstance(comp, CurrentSourceDC):
            comp.current = comp.value # Current is defined by the source
        elif isinstance(comp, VoltageSourceDC):
            # Current through a voltage source is not directly solved by this Nodal method.
            comp.current = None # Mark as not calculated by this solver phase

    # print("Solved Node Voltages:", circuit.solved_node_voltages) # For debugging
    # for comp in circuit.components: # For debugging
    #     print(f"Component {comp.id}: V={comp.voltage}, I={comp.current}")


def get_symbolic_voltage_for_node(
    node_name: str,
    circuit_ground_node: str | None,
    defined_node_voltages_map: dict[str, sympy.Expr]
) -> sympy.Expr:
    """
    Returns the symbolic voltage expression for a given node.
    - Ground node is 0.
    - Nodes fixed by voltage sources use their source's symbolic value.
    - Other non-ground nodes use their generic V_Node symbol.
    """
    if node_name == circuit_ground_node:
        return sympy.Integer(0)
    
    if node_name in defined_node_voltages_map:
        return defined_node_voltages_map[node_name]
    else:
        # This should ideally not be reached if defined_node_voltages_map is pre-populated correctly
        # with all non-ground node symbols (like V_N1, V_N2 etc.)
        # or if node_name is ground (handled above).
        raise KeyError(f"Symbolic voltage for node '{node_name}' not found in internal map. "
                       "Ensure it's a non-ground node or properly handled as ground.")


def formulate_symbolic_dc_equations(circuit: Circuit) -> tuple[
    dict[str, sympy.Eq],         # Node name to KCL Equation (for unknown nodes)
    dict[str, sympy.Eq],         # Node name to Explicit Voltage Definition (e.g. V_N1 = V_S1)
    dict[str, sympy.Symbol],     # Node name to original V_Node symbol (all non-ground)
    dict[str, sympy.Symbol]      # Component ID to component value symbol
    ]:
    """
    Formulates symbolic KCL equations for a DC circuit based on standard Nodal Analysis.

    Returns:
        - kcl_equations_for_unknowns: Dict mapping node names (whose voltages are unknown) to their KCL sympy.Eq.
        - explicit_voltage_definitions: Dict mapping node names (fixed by sources) to their definition Eq (e.g. V_N1 = V_S1).
        - base_node_voltage_symbols: Dict of all non-ground node base voltage symbols (e.g., V_N1).
        - component_value_symbols: Dict of all component value symbols (e.g., R_R1).
    """
    if circuit.ground_node is None:
        raise ValueError("Circuit must have a ground node set for symbolic formulation.")
    if not circuit.node_map and (circuit.nodes - {circuit.ground_node}):
        # Node map is essential. Try to build if not present but nodes exist.
        try:
            print("Symbolic formulation: Node map not found, attempting to build...")
            circuit.build_node_map()
            if not circuit.node_map and (circuit.nodes - {circuit.ground_node}): # Check again
                raise ValueError("Node map is still empty despite non-ground nodes present.")
        except ValueError as e:
            raise ValueError(f"Error building node map before symbolic formulation: {e}")
    
    # Handle cases like only ground node or empty circuit
    if not circuit.node_map:
        return {}, {}, {}, {}

    base_node_voltage_symbols = create_node_voltage_symbols(circuit)
    component_value_symbols = create_component_value_symbols(circuit)

    # This map will store the actual symbolic expression to use for each node's voltage.
    # Initially, it's the base V_NodeX symbol for all non-ground nodes.
    # It gets updated if a node's voltage is fixed by a source.
    effective_node_voltage_expr_map: dict[str, sympy.Expr] = {
        name: sym for name, sym in base_node_voltage_symbols.items()
    }

    explicit_voltage_definitions: dict[str, sympy.Eq] = {}

    # Step 1: Identify nodes fixed by grounded voltage sources
    for comp in circuit.components:
        if isinstance(comp, VoltageSourceDC):
            n_pos, n_neg = comp.nodes[0], comp.nodes[1]
            source_val_sym = component_value_symbols[comp.id]
            
            is_n_pos_ground = (n_pos == circuit.ground_node)
            is_n_neg_ground = (n_neg == circuit.ground_node)

            target_node_name: str | None = None
            voltage_expression: sympy.Expr | None = None

            if not is_n_pos_ground and is_n_neg_ground:
                target_node_name = n_pos
                voltage_expression = source_val_sym
            elif not is_n_neg_ground and is_n_pos_ground:
                target_node_name = n_neg
                voltage_expression = -source_val_sym
            elif not is_n_pos_ground and not is_n_neg_ground:
                raise NotImplementedError(
                    f"Floating voltage source {comp.id} ({n_pos}-{n_neg}) is not supported "
                    "for this symbolic KCL formulation (requires MNA)."
                )
            # If both are ground, no non-ground node voltage is defined by this source.

            if target_node_name and voltage_expression is not None:
                if target_node_name not in base_node_voltage_symbols:
                    # This implies a VSource is connected to a node not otherwise in the circuit,
                    # or the node_map is incomplete.
                    raise ValueError(f"Voltage source {comp.id} connects to '{target_node_name}', "
                                     "which is not in the set of non-ground nodes. Circuit definition error.")

                # Check for conflicting definitions
                current_expr = effective_node_voltage_expr_map.get(target_node_name)
                base_sym = base_node_voltage_symbols.get(target_node_name)
                if current_expr != base_sym and current_expr is not None: # Already defined by another source
                     print(f"Warning: Voltage for node '{target_node_name}' (currently {current_expr}) "
                           f"is being redefined by source '{comp.id}' to {voltage_expression}.")

                effective_node_voltage_expr_map[target_node_name] = voltage_expression
                explicit_voltage_definitions[target_node_name] = sympy.Eq(
                    base_node_voltage_symbols[target_node_name], voltage_expression
                )

    # Step 2: Formulate KCL for each non-ground node whose voltage is NOT explicitly defined by a source
    kcl_equations_for_unknowns: dict[str, sympy.Eq] = {} 
    
    for node_k_name in circuit.node_map.keys():
        if node_k_name in explicit_voltage_definitions:
            # This node's voltage is already defined by a source.
            # Standard Nodal Analysis does not write a KCL *to solve for this node's voltage*.
            continue

        current_sum_at_node_k = sympy.Integer(0)

        for comp in circuit.components:
            if node_k_name not in comp.nodes:
                # Component not connected to the current KCL node
                continue

            n1_name, n2_name = comp.nodes[0], comp.nodes[1]
            comp_val_sym = component_value_symbols[comp.id]

            v1_sym_expr = get_symbolic_voltage_for_node(n1_name, circuit.ground_node, effective_node_voltage_expr_map)
            v2_sym_expr = get_symbolic_voltage_for_node(n2_name, circuit.ground_node, effective_node_voltage_expr_map)

            if isinstance(comp, Resistor):
                # Current leaving node_k_name
                if n1_name == node_k_name: 
                    current_sum_at_node_k += (v1_sym_expr - v2_sym_expr) / comp_val_sym
                elif n2_name == node_k_name:
                    current_sum_at_node_k += (v2_sym_expr - v1_sym_expr) / comp_val_sym
            
            elif isinstance(comp, CurrentSourceDC):
                # Positive comp_val_sym means current flows n1->n2 through source.
                # KCL: sum of currents leaving node_k_name = 0.
                if n1_name == node_k_name: # Current leaves node_k_name via the source's n1 terminal
                    current_sum_at_node_k += comp_val_sym 
                elif n2_name == node_k_name: # Current enters node_k_name via the source's n2 terminal
                    current_sum_at_node_k -= comp_val_sym
            
            elif isinstance(comp, VoltageSourceDC):
                # Effect of grounded VSources is already incorporated via effective_node_voltage_expr_map.
                # They don't contribute a separate current term here in this Nodal formulation.
                # Floating VSources would have raised an error.
                pass

        kcl_equations_for_unknowns[node_k_name] = sympy.Eq(current_sum_at_node_k, 0)
    
    return (
        kcl_equations_for_unknowns,
        explicit_voltage_definitions,
        base_node_voltage_symbols,
        component_value_symbols
    )

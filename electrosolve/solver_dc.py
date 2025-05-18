import numpy as np
from .circuit import Circuit, Resistor, VoltageSourceDC, CurrentSourceDC

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

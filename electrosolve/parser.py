import json
from .circuit import Circuit, Resistor, VoltageSourceDC, CurrentSourceDC, Component

def load_circuit_from_json(file_path: str) -> Circuit:
    """
    Loads a circuit definition from a JSON file and returns a Circuit object.

    The JSON format should be a dictionary with keys like "components" and "ground_node".
    "components" should be a list of component dictionaries.
    Each component dictionary should have "id", "type", "value", and "nodes".

    Example component:
    {
        "id": "R1",
        "type": "resistor",
        "value": 1000,
        "nodes": ["N1", "GND"]
    }
    """
    try:
        with open(file_path, 'r') as f:
            data = json.load(f)
    except FileNotFoundError:
        raise FileNotFoundError(f"Error: Circuit file '{file_path}' not found.")
    except json.JSONDecodeError:
        raise ValueError(f"Error: Invalid JSON format in '{file_path}'.")

    circuit = Circuit()

    if "components" not in data:
        raise ValueError("JSON circuit definition must contain a 'components' list.")
    if not isinstance(data["components"], list):
        raise ValueError("'components' must be a list.")

    component_ids = set()

    for comp_data in data["components"]:
        if not isinstance(comp_data, dict):
            raise ValueError(f"Each item in 'components' list must be a dictionary. Found: {comp_data}")

        required_keys = {"id", "type", "value", "nodes"}
        if not required_keys.issubset(comp_data.keys()):
            missing_keys = required_keys - comp_data.keys()
            raise ValueError(f"Component data missing required keys: {missing_keys}. Data: {comp_data}")

        comp_id = comp_data["id"]
        comp_type = comp_data["type"].lower()

        if comp_id in component_ids:
            raise ValueError(f"Duplicate component ID '{comp_id}' found. IDs must be unique.")
        component_ids.add(comp_id)

        try:
            comp_value = comp_data["value"]
            if not isinstance(comp_value, (int, float)):
                raise ValueError(f"Component '{comp_id}' value must be a number. Got: {comp_value}")
        except KeyError:
            raise ValueError(f"Component '{comp_id}' is missing 'value' field.")


        comp_nodes = comp_data["nodes"]
        if not isinstance(comp_nodes, list) or len(comp_nodes) != 2:
            raise ValueError(
                f"Component '{comp_id}' nodes must be a list of two strings. Got: {comp_nodes}"
            )
        if not all(isinstance(n, str) for n in comp_nodes):
            raise ValueError(
                f"Component '{comp_id}' node names must be strings. Got: {comp_nodes}"
            )


        component: Component | None = None
        if comp_type == "resistor":
            component = Resistor(id=comp_id, value=float(comp_value), nodes=comp_nodes)
        elif comp_type == "voltagesourcedc": # Or just "voltagesource" if we want to be less specific for now
            component = VoltageSourceDC(id=comp_id, value=float(comp_value), nodes=comp_nodes)
        elif comp_type == "currentsourcedc": # Or "currentsource"
            component = CurrentSourceDC(id=comp_id, value=float(comp_value), nodes=comp_nodes)
        else:
            raise ValueError(f"Unknown component type '{comp_type}' for component '{comp_id}'.")

        circuit.add_component(component)

    # Set ground node
    if "ground_node" not in data:
        raise ValueError("JSON circuit definition must specify a 'ground_node'.")

    ground_node_id = data["ground_node"]
    if not isinstance(ground_node_id, str):
        raise ValueError("'ground_node' must be a string.")

    # Ensure ground node is one of the nodes defined by components, or explicitly add it.
    # The Circuit.set_ground_node method handles whether it needs to exist already.
    # For robustness, let's ensure it was mentioned by at least one component or as the ground itself.
    if ground_node_id not in circuit.nodes:
        # This check can be debated. `circuit.set_ground_node` currently adds it.
        # If we want to be strict that ground must be part of a component:
        # raise ValueError(f"Specified ground node '{ground_node_id}' is not connected to any component.")
        pass # circuit.set_ground_node will add it to circuit.nodes if not present

    circuit.set_ground_node(ground_node_id)

    try:
        circuit.build_node_map()
    except ValueError as e:
        raise ValueError(f"Error building node map for circuit: {e}")

    return circuit

import numpy as np
import sympy


class Component:
    def __init__(self, id: str, value: float, nodes: list[str]):
        """
        Base class for a circuit component.

        Args:
            id (str): A unique identifier for the component (e.g., "R1", "V_source").
            value (float): The numerical value of the component (e.g., Ohms, Volts, Amps).
            nodes (list[str]): A list of two node IDs to which the component is connected.
                               For sources, the order can indicate polarity/direction.
        """
        self.id = id
        self.value = float(value)
        self.nodes = nodes

        if len(nodes) != 2:
            # For now, we'll assume all components are two-terminal.
            # This can be generalized later if needed (e.g., for transistors).
            raise ValueError(f"Component '{id}' must connect to exactly two nodes. Got: {nodes}")

        self.voltage = None
        self.current = None

    def __repr__(self):
        return (f"{self.__class__.__name__}(id='{self.id}', value={self.value}, "
                f"nodes={self.nodes}, voltage={self.voltage}, current={self.current})")

    def get_node_indices(self, node_map: dict[str, int], ground_node: str) -> tuple[int | None, int | None]:
        """
        Helper to get integer indices for component nodes based on the circuit's node_map.
        Returns None for a ground node index as it's not part of the G matrix directly.
        """
        node1_idx = node_map.get(self.nodes[0]) if self.nodes[0] != ground_node else -1
        node2_idx = node_map.get(self.nodes[1]) if self.nodes[1] != ground_node else -1
        return node1_idx, node2_idx

class Resistor(Component):
    def __init__(self, id: str, value: float, nodes: list[str]):
        super().__init__(id, value, nodes)
        if self.value <= 0:
            raise ValueError(f"Resistor '{self.id}' must have a positive value. Got: {self.value}")

class VoltageSourceDC(Component):
    def __init__(self, id: str, value: float, nodes: list[str]):
        """
        DC Voltage Source.
        Convention: nodes[0] is the positive terminal, nodes[1] is the negative terminal.
        Voltage is V(nodes[0]) - V(nodes[1]) = value.
        """
        super().__init__(id, value, nodes)

class CurrentSourceDC(Component):
    def __init__(self, id: str, value: float, nodes: list[str]):
        """
        DC Current Source.
        Convention: Current flows from nodes[0] to nodes[1] *through the external circuit*.
        Thus, current enters the circuit at nodes[1] and leaves at nodes[0] (if thinking about the source itself).
        Or, more simply for nodal analysis: current 'value' is injected into nodes[1] and drawn from nodes[0].
        Let's refine: positive value means current flows from nodes[0] to nodes[1] *through the source itself*.
        So, current enters node nodes[0] and exits node nodes[1] in the G matrix formulation.
        Let's stick to the common SPICE convention: current flows from positive node (nodes[0]) to negative node (nodes[1]) *through the source*.
        So, for nodal analysis, `Is` is *added* to the KCL equation of `nodes[1]` and *subtracted* from `nodes[0]`.
        The plan: "Current `Is` flowing into node `j` and out of node `k` => `I[j] -= Is`, `I[k] += Is`".
        This implies current source value is directed from node `k` to node `j`.
        Let's define it as: A positive `value` means current flows from `nodes[0]` to `nodes[1]` (externally).
        This means `value` current is *supplied to* `nodes[1]` and *drawn from* `nodes[0]`.
        So, `I[node_map[nodes[0]]] -= value` and `I[node_map[nodes[1]]] += value`.
        """
        super().__init__(id, value, nodes)

class Circuit:
    def __init__(self):
        self.components: list[Component] = []
        self.nodes: set[str] = set()
        self.ground_node: str | None = None
        self.node_map: dict[str, int] = {}
        self.num_non_ground_nodes: int = 0
        self.solution_v: np.ndarray | None = None
        self.solved_node_voltages: dict[str, float] = {}

    def add_component(self, component: Component):
        """Adds a component to the circuit and registers its nodes."""
        if not isinstance(component, Component):
            raise TypeError("Can only add Component objects to the circuit.")
        self.components.append(component)
        for node_id in component.nodes:
            self.nodes.add(node_id)

    def set_ground_node(self, node_id: str):
        """Sets the ground node for the circuit."""
        if node_id not in self.nodes:
            self.nodes.add(node_id)
        if self.ground_node is not None and self.ground_node != node_id:
            raise ValueError(f"Ground node already set to '{self.ground_node}'. Cannot re-assign to '{node_id}'.")
        self.ground_node = node_id
        print(f"Circuit: Ground node set to '{self.ground_node}'")


    def build_node_map(self):
        """
        Builds a mapping from non-ground node names to matrix indices (0 to N-1).
        This must be called after all components are added and ground_node is set.
        """
        if self.ground_node is None:
            raise ValueError("Ground node must be set before building the node map.")
        if not self.nodes:
            print("Warning: No components added to the circuit yet.")
            self.node_map = {}
            self.num_non_ground_nodes = 0
            return

        non_ground_nodes_sorted = sorted(list(self.nodes - {self.ground_node}))
        self.node_map = {node_id: i for i, node_id in enumerate(non_ground_nodes_sorted)}
        self.num_non_ground_nodes = len(non_ground_nodes_sorted)
        print(f"Circuit: Node map built. Non-ground nodes ({self.num_non_ground_nodes}): {self.node_map}")

    def get_component(self, component_id: str) -> Component | None:
        """Retrieves a component by its ID."""
        for comp in self.components:
            if comp.id == component_id:
                return comp
        return None

    def get_component_by_symbol(self, target_symbol: sympy.Symbol, component_value_symbols: dict[str, sympy.Symbol]) -> Component | None:
        """
        Retrieves a component whose value symbol matches the target_symbol.
        """
        for comp_id, sym in component_value_symbols.items():
            if sym == target_symbol:
                return self.get_component(comp_id)
        return None

    def __repr__(self):
        return (f"Circuit(components={len(self.components)}, nodes={self.nodes}, "
                f"ground_node='{self.ground_node}', num_non_ground_nodes={self.num_non_ground_nodes})")

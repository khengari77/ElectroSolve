# electrosolve/tests/test_parser.py
import unittest
import json
import os
import tempfile
from electrosolve.parser import load_circuit_from_json
from electrosolve.circuit import Circuit, Resistor, VoltageSourceDC, CurrentSourceDC

class TestJSONParser(unittest.TestCase):

    def create_temp_json_file(self, data):
        """Helper to create a temporary JSON file with given data."""
        temp_file = tempfile.NamedTemporaryFile(mode='w+', delete=False, suffix='.json')
        json.dump(data, temp_file)
        temp_file.close()
        return temp_file.name

    def tearDown(self):
        if hasattr(self, 'temp_file_path') and self.temp_file_path:
            if os.path.exists(self.temp_file_path):
                os.unlink(self.temp_file_path)

    def test_load_valid_simple_dc_circuit(self):
        """Test loading a valid simple DC circuit from a JSON structure."""
        valid_circuit_data = {
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
                    "value": 100.0,
                    "nodes": ["N1", "N2"]
                },
                {
                    "id": "R2",
                    "type": "resistor",
                    "value": 200.0,
                    "nodes": ["N2", "GND"]
                },
                {
                    "id": "Is1",
                    "type": "currentsourcedc",
                    "value": 0.1,
                    "nodes": ["N2", "GND"] # Current 0.1A from N2 to GND (source convention)
                                          # Meaning 0.1A injected into GND, drawn from N2
                                          # Let's refine parser.py convention.
                                          # Plan: "Is flowing into node j and out of node k => I[j] -= Is, I[k] += Is"
                                          # This implies source value is directed from k to j.
                                          # Circuit.py CurrentSourceDC comment:
                                          # "positive value means current flows from nodes[0] to nodes[1] *through the source itself*."
                                          # So for KCL, current is added to KCL of nodes[1] and subtracted from KCL of nodes[0].
                                          # For Is1: nodes=["N2", "GND"], value=0.1A -> 0.1A flows from N2 to GND through the source.
                                          # This means 0.1A is injected into the GND node by Is1 and drawn from N2 node by Is1.
                                          # For KCL at N2: -0.1A. For KCL at GND (not formed): +0.1A. This seems consistent.
                }
            ],
            "ground_node": "GND"
        }

        self.temp_file_path = self.create_temp_json_file(valid_circuit_data)

        circuit = load_circuit_from_json(self.temp_file_path)

        self.assertIsInstance(circuit, Circuit)
        self.assertEqual(len(circuit.components), 4)
        self.assertEqual(circuit.ground_node, "GND")
        self.assertEqual(circuit.num_non_ground_nodes, 2) # N1, N2
        self.assertIn("N1", circuit.node_map)
        self.assertIn("N2", circuit.node_map)
        self.assertNotIn("GND", circuit.node_map)

        # Check component details
        vs1 = circuit.get_component("Vs1")
        self.assertIsInstance(vs1, VoltageSourceDC)
        self.assertEqual(vs1.value, 9.0)
        self.assertEqual(vs1.nodes, ["N1", "GND"])

        r1 = circuit.get_component("R1")
        self.assertIsInstance(r1, Resistor)
        self.assertEqual(r1.value, 100.0)
        self.assertEqual(r1.nodes, ["N1", "N2"])

        r2 = circuit.get_component("R2")
        self.assertIsInstance(r2, Resistor)
        self.assertEqual(r2.value, 200.0)
        self.assertEqual(r2.nodes, ["N2", "GND"])

        is1 = circuit.get_component("Is1")
        self.assertIsInstance(is1, CurrentSourceDC)
        self.assertEqual(is1.value, 0.1)
        self.assertEqual(is1.nodes, ["N2", "GND"]) # As defined in JSON

        os.unlink(self.temp_file_path) # Clean up
        self.temp_file_path = None


    def test_missing_components_key(self):
        invalid_data = {"ground_node": "GND"}
        self.temp_file_path = self.create_temp_json_file(invalid_data)
        with self.assertRaisesRegex(ValueError, "must contain a 'components' list"):
            load_circuit_from_json(self.temp_file_path)
        os.unlink(self.temp_file_path)
        self.temp_file_path = None

    def test_missing_ground_node_key(self):
        invalid_data = {"components": []}
        self.temp_file_path = self.create_temp_json_file(invalid_data)
        with self.assertRaisesRegex(ValueError, "must specify a 'ground_node'"):
            load_circuit_from_json(self.temp_file_path)
        os.unlink(self.temp_file_path)
        self.temp_file_path = None

    def test_component_missing_type(self):
        invalid_data = {
            "components": [{"id": "R1", "value": 100, "nodes": ["N1", "GND"]}],
            "ground_node": "GND"
        }
        self.temp_file_path = self.create_temp_json_file(invalid_data)
        with self.assertRaisesRegex(ValueError, "missing required keys: {'type'}"):
            load_circuit_from_json(self.temp_file_path)
        os.unlink(self.temp_file_path)
        self.temp_file_path = None

    def test_unknown_component_type(self):
        invalid_data = {
            "components": [{"id": "X1", "type": "diode", "value": 0.7, "nodes": ["N1", "GND"]}],
            "ground_node": "GND"
        }
        self.temp_file_path = self.create_temp_json_file(invalid_data)
        with self.assertRaisesRegex(ValueError, "Unknown component type 'diode'"):
            load_circuit_from_json(self.temp_file_path)
        os.unlink(self.temp_file_path)
        self.temp_file_path = None

    def test_duplicate_component_id(self):
        invalid_data = {
            "components": [
                {"id": "R1", "type": "resistor", "value": 100, "nodes": ["N1", "GND"]},
                {"id": "R1", "type": "resistor", "value": 200, "nodes": ["N2", "GND"]}
            ],
            "ground_node": "GND"
        }
        self.temp_file_path = self.create_temp_json_file(invalid_data)
        with self.assertRaisesRegex(ValueError, "Duplicate component ID 'R1' found"):
            load_circuit_from_json(self.temp_file_path)
        os.unlink(self.temp_file_path)
        self.temp_file_path = None

    def test_invalid_node_format(self):
        invalid_data = {
            "components": [{"id": "R1", "type": "resistor", "value": 100, "nodes": "N1-GND"}], # nodes should be a list
            "ground_node": "GND"
        }
        self.temp_file_path = self.create_temp_json_file(invalid_data)
        with self.assertRaisesRegex(ValueError, "nodes must be a list of two strings"):
            load_circuit_from_json(self.temp_file_path)
        os.unlink(self.temp_file_path)
        self.temp_file_path = None

    def test_component_value_not_number(self):
        invalid_data = {
            "components": [{"id": "R1", "type": "resistor", "value": "100k", "nodes": ["N1", "GND"]}],
            "ground_node": "GND"
        }
        self.temp_file_path = self.create_temp_json_file(invalid_data)
        with self.assertRaisesRegex(ValueError, "Component 'R1' value must be a number. Got: 100k"):
            load_circuit_from_json(self.temp_file_path)
        os.unlink(self.temp_file_path)
        self.temp_file_path = None

if __name__ == '__main__':
    unittest.main()

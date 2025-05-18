import unittest
import numpy as np
from electrosolve.circuit import Circuit, Resistor, VoltageSourceDC, CurrentSourceDC
from electrosolve.solver_dc import solve_dc_circuit

class TestDCCircuitSolver(unittest.TestCase):

    def assertAlmostEqualNodeVoltages(self, calculated_voltages, expected_voltages, places=7):
        """Helper to compare dictionaries of node voltages."""
        self.assertEqual(len(calculated_voltages), len(expected_voltages), "Number of nodes differs.")
        for node, expected_v in expected_voltages.items():
            self.assertIn(node, calculated_voltages, f"Node {node} not in calculated voltages.")
            self.assertAlmostEqual(calculated_voltages[node], expected_v, places=places,
                                   msg=f"Voltage at node {node} differs.")

    def test_simple_voltage_divider(self):
        """Tests a simple voltage divider circuit (Vs - R1 - R2 - GND)."""
        circuit = Circuit()
        vs1 = VoltageSourceDC(id="Vs1", value=9.0, nodes=["N1", "GND"])
        r1 = Resistor(id="R1", value=1000.0, nodes=["N1", "N2"])
        r2 = Resistor(id="R2", value=2000.0, nodes=["N2", "GND"])

        circuit.add_component(vs1)
        circuit.add_component(r1)
        circuit.add_component(r2)
        circuit.set_ground_node("GND")
        circuit.build_node_map()

        solve_dc_circuit(circuit)

        expected_voltages = {
            "GND": 0.0,
            "N1": 9.0,
            "N2": 6.0
        }
        self.assertAlmostEqualNodeVoltages(circuit.solved_node_voltages, expected_voltages)

        # Check component values
        comp_vs1 = circuit.get_component("Vs1")
        self.assertAlmostEqual(comp_vs1.voltage, 9.0) # V_N1 - V_GND
        self.assertIsNone(comp_vs1.current) # Not calculated by this solver

        comp_r1 = circuit.get_component("R1")
        self.assertAlmostEqual(comp_r1.voltage, 3.0)  # V_N1 - V_N2 = 9.0 - 6.0
        self.assertAlmostEqual(comp_r1.current, 0.003) # 3.0 / 1000.0

        comp_r2 = circuit.get_component("R2")
        self.assertAlmostEqual(comp_r2.voltage, 6.0)  # V_N2 - V_GND = 6.0 - 0.0
        self.assertAlmostEqual(comp_r2.current, 0.003) # 6.0 / 2000.0

    def test_current_source_with_resistor(self):
        """Tests a current source feeding a resistor to ground."""
        circuit = Circuit()
        # Is1: 0.01A flows from N1 to GND through the source.
        # This means 0.01A is drawn from N1.
        is1 = CurrentSourceDC(id="Is1", value=0.01, nodes=["N1", "GND"])
        r1 = Resistor(id="R1", value=500.0, nodes=["N1", "GND"])

        circuit.add_component(is1)
        circuit.add_component(r1)
        circuit.set_ground_node("GND")
        circuit.build_node_map()

        solve_dc_circuit(circuit)
        
        # KCL at N1 (sum of currents leaving N1 = 0):
        # Current leaving N1 via Is1: +0.01A
        # Current leaving N1 via R1: V_N1 / 500.0
        # 0.01 + V_N1 / 500.0 = 0  => V_N1 = -0.01 * 500.0 = -5.0V
        expected_voltages = {
            "GND": 0.0,
            "N1": -5.0
        }
        self.assertAlmostEqualNodeVoltages(circuit.solved_node_voltages, expected_voltages)

        comp_is1 = circuit.get_component("Is1")
        self.assertAlmostEqual(comp_is1.voltage, -5.0) # V_N1 - V_GND = -5.0 - 0.0
        self.assertAlmostEqual(comp_is1.current, 0.01) # Defined by source

        comp_r1 = circuit.get_component("R1")
        self.assertAlmostEqual(comp_r1.voltage, -5.0)   # V_N1 - V_GND = -5.0 - 0.0
        self.assertAlmostEqual(comp_r1.current, -0.01) # -5.0V / 500Ohms

    def test_circuit_with_multiple_sources(self):
        """ Vs1 - R1 - N2 - R2 - GND, with Is1 also feeding N2 from N3 (N3 connected to Vs2 to GND) """
        circuit = Circuit()
        circuit.add_component(VoltageSourceDC(id="Vs1", value=10.0, nodes=["N1", "GND"]))
        circuit.add_component(Resistor(id="R1", value=100.0, nodes=["N1", "N2"]))
        circuit.add_component(Resistor(id="R2", value=200.0, nodes=["N2", "GND"]))
        # Is1: 0.02A from N2 to N3 through source. Draws 0.02A from N2.
        circuit.add_component(CurrentSourceDC(id="Is1", value=0.02, nodes=["N2", "N3"]))
        circuit.add_component(VoltageSourceDC(id="Vs2", value=5.0, nodes=["N3", "GND"]))
        circuit.set_ground_node("GND")
        circuit.build_node_map()

        solve_dc_circuit(circuit)

        # Expected hand calculations:
        # V_N1 = 10V (from Vs1)
        # V_N3 = 5V  (from Vs2)
        # KCL at N2 (sum of currents leaving N2 = 0):
        # (V_N2 - V_N1)/R1 + V_N2/R2 + Is1.value = 0
        # (V_N2 - 10)/100 + V_N2/200 + 0.02 = 0
        # 0.01*V_N2 - 0.1 + 0.005*V_N2 + 0.02 = 0
        # 0.015*V_N2 - 0.08 = 0
        # 0.015*V_N2 = 0.08
        # V_N2 = 0.08 / 0.015 = 16.0/3.0 (approx 5.3333...V)

        expected_voltages = {
            "GND": 0.0,
            "N1": 10.0,
            "N2": 16.0/3.0,
            "N3": 5.0
        }
        self.assertAlmostEqualNodeVoltages(circuit.solved_node_voltages, expected_voltages, places=5)

        v_n1_calc = 10.0
        v_n2_calc = 16.0/3.0
        v_n3_calc = 5.0

        comp_r1 = circuit.get_component("R1") # N1-N2
        self.assertAlmostEqual(comp_r1.voltage, v_n1_calc - v_n2_calc) 
        self.assertAlmostEqual(comp_r1.current, (v_n1_calc - v_n2_calc) / 100.0)

        comp_r2 = circuit.get_component("R2") # N2-GND
        self.assertAlmostEqual(comp_r2.voltage, v_n2_calc - 0.0) 
        self.assertAlmostEqual(comp_r2.current, v_n2_calc / 200.0)

        comp_is1 = circuit.get_component("Is1") # N2-N3
        self.assertAlmostEqual(comp_is1.voltage, v_n2_calc - v_n3_calc) 
        self.assertAlmostEqual(comp_is1.current, 0.02)


    def test_singular_matrix_floating_resistor(self):
        """Tests a circuit that would result in a singular matrix (e.g., floating component)."""
        # This test case primarily focuses on circuit_singular.
        # The initial `circuit` variable defines a more complex circuit which would also be singular
        # due to N_float1 and N_float2, but it's not directly asserted upon.
        
        # circuit = Circuit()
        # circuit.add_component(VoltageSourceDC(id="Vs1", value=5.0, nodes=["N1", "GND"]))
        # circuit.add_component(Resistor(id="R_float", value=100.0, nodes=["N_float1", "N_float2"]))
        # circuit.add_component(Resistor(id="R1", value=100.0, nodes=["N1", "N_other"]))
        # circuit.add_component(Resistor(id="R2", value=100.0, nodes=["N_other", "GND"]))
        # circuit.set_ground_node("GND")
        # with self.assertRaisesRegex(RuntimeError, "Failed to solve circuit: Linear algebra error"):
        #     solve_dc_circuit(circuit) # This should also fail

        # Simpler singular case: R1 between A and B, nothing else. No ground ref for A/B.
        circuit_singular = Circuit()
        circuit_singular.add_component(Resistor(id="R1", value=100.0, nodes=["A", "B"]))
        # Set a ground node that is not connected to "A" or "B", making them floating.
        circuit_singular.set_ground_node("GND_unconnected") 

        with self.assertRaisesRegex(RuntimeError, "Failed to solve circuit: Linear algebra error"):
            # build_node_map() will be called by solver_dc.solve_dc_circuit if needed
            solve_dc_circuit(circuit_singular)

    def test_voltage_source_short(self):
        """ Test two voltage sources in parallel (a conflict if different values) """
        circuit = Circuit()
        circuit.add_component(VoltageSourceDC("Vs1", 9.0, ["N1", "GND"]))
        circuit.add_component(VoltageSourceDC("Vs2", 5.0, ["N1", "GND"])) # Vs1 and Vs2 fight over V(N1)
        circuit.set_ground_node("GND")
        # circuit.build_node_map() # Called by solver

        # The current solver updates G and I. The last VSource setting V(N1) will win.
        # This isn't a LinAlgError from G being singular, but rather an ill-defined circuit.
        # The solver will produce a result based on the last modification.
        solve_dc_circuit(circuit)
        # Vs2 is added last, so its definition for N1 (5.0V) should prevail.
        self.assertAlmostEqual(circuit.solved_node_voltages["N1"], 5.0)
        # This highlights a limitation: the solver doesn't detect semantic errors like this yet.

if __name__ == '__main__':
    unittest.main()

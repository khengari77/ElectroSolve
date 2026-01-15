use crate::errors::ParseError;
use crate::graph::{CircuitGraph, NodeIndex};
use crate::component::{ComponentKind};
use crate::units::{Voltage, Current, Resistance, Inductance, Capacitance};
use std::collections::HashMap;
use std::fs;

pub fn parse_value(input: &str, line_num: usize) -> Result<f64, ParseError> {
    let input = input.trim();

    let suffix_start = input
            .find(|c: char| !c.is_ascii_digit() && c != '.' && c != '-')
            .unwrap_or(input.len());

    let numeric_part = &input[..suffix_start];
    let suffix = &input[suffix_start..].trim();

    let value: f64 = numeric_part.parse()
        .map_err(|_| ParseError { line: line_num, message: format!("Invalid character: {numeric_part}") })?;

    let multiplier = match suffix.to_lowercase().as_str() {
        "t" => 1e12,
        "g" => 1e9,
        "meg" => 1e6,
        "k" => 1e3,
        "m" => 1e-3,
        "u" | "µ" => 1e-6,
        "n" => 1e-9,
        "p" => 1e-12,
        "" => 1.0,
        _ => return Err(ParseError { line: line_num, message: format!("Invalid suffix: {suffix}") }),
    };
    Ok(value * multiplier)
}

pub fn get_or_create_node(name: &str, graph: &mut CircuitGraph, node_map: &mut HashMap<String, NodeIndex>) -> NodeIndex {
    if let Some(idx) = node_map.get(name) {
        return *idx;
    }

    let idx = graph.add_node(name.to_string());
    node_map.insert(name.to_string(), idx);
    idx
}

pub fn parse_component_line(
    line: &str, 
    line_num: usize, 
    graph: &mut CircuitGraph,
    node_map: &mut HashMap<String, NodeIndex>
) -> Result<(), ParseError> {
    let line = line.trim();
    if line.is_empty() || line.starts_with('*') {
        return Ok(());
    }
    let tokens: Vec<&str> = line.split_whitespace().collect();
    if tokens.len() < 4 {
        return Err(ParseError { line: line_num, message: format!("Expected at least 4 tokens, got {tokens:?}") });
    }
    let component_id = tokens[0];
    let first_char = component_id.chars().next()
        .ok_or_else(|| ParseError { line: line_num, message: format!("Invalid component ID: {component_id}") })?;

    let node1_name = tokens[1];
    let node2_name = tokens[2];

    let node1_idx = get_or_create_node(node1_name, graph, node_map);
    let node2_idx = get_or_create_node(node2_name, graph, node_map);
    let value = parse_value(tokens[3], line_num)?;

    let kind = match first_char {
        'R' => ComponentKind::Resistor { r: Resistance::known(value)? },
        'L' => ComponentKind::Inductor { l: Inductance::known(value)? },
        'C' => ComponentKind::Capacitor { c: Capacitance::known(value)? },
        'V' => {
            // Check for AC/DC specification
            let v = if tokens.len() >= 5 && tokens[4].to_uppercase() == "AC" {
                // AC voltage: V1 N1 0 AC <magnitude> <phase>
                let magnitude = if tokens.len() >= 6 {
                    parse_value(tokens[5],  line_num)?
                } else {
                    value  // Use same value as default
                };
                let phase = if tokens.len() >= 7 {
                    parse_value(tokens[6],  line_num)?
                } else {
                    0.0
                };
                Voltage::ac_phasor(magnitude, phase)
            } else {
                // DC voltage
                Voltage::dc(value)
            };
            ComponentKind::VoltageSource { v }
        }
        'I' => {
            // Check for AC/DC specification
            let i = if tokens.len() >= 5 && tokens[4].to_uppercase() == "AC" {
                // AC current: I1 N1 0 AC <magnitude> <phase>
                let magnitude = if tokens.len() >= 6 {
                    parse_value(tokens[5],  line_num)?
                } else {
                    value  // Use same value as default
                };
                let phase = if tokens.len() >= 7 {
                    parse_value(tokens[6], line_num)?
                } else {
                    0.0
                };
                Current::ac_phasor(magnitude, phase)
            } else {
                // DC current
                Current::dc(value)
            };
            ComponentKind::CurrentSource { i }
        },
        _ => return Err(ParseError { line: line_num, message: format!("Unknown component type: {first_char}") }),
    };
    graph.add_component(component_id.to_string(), kind, (node1_idx, node2_idx));
    if node1_name.to_lowercase() == "gnd" || node1_name == "0" {
        graph.set_ground(node1_idx);
    }
    if node2_name.to_lowercase() == "gnd" || node2_name == "0" {
        graph.set_ground(node2_idx);
    }
    Ok(())
}

pub fn parse_netlist(input: &str) -> Result<CircuitGraph, ParseError> {
    let mut graph = CircuitGraph::new();
    let mut node_map = HashMap::new();
    for (line_num, line) in input.lines().enumerate() {
        let line_num = line_num + 1;
        if let Err(e) = parse_component_line(line, line_num, &mut graph, &mut node_map) {
            return Err(e);
        }   
    }

    if graph.ground.is_none() {
        return Err(ParseError { line: 0, message: "No ground node specified".to_string() });
    }
    if graph.components.is_empty() {
        return Err(ParseError { line: 0, message: "No components specified".to_string() });
    }
    Ok(graph)
}

pub fn parse_file(path: &str) -> Result<CircuitGraph, ParseError> {
    let contents = fs::read_to_string(path).map_err(|_| ParseError { line: 0, message: format!("Failed to read file: {path}") })?;
    parse_netlist(&contents)
}

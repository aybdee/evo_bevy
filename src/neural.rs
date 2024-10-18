// use crate::gene::Genome;
use bevy::reflect::Map;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

pub const WEIGHT_RANGE: f32 = 4.0;

#[derive(Clone)]
pub struct NeuralNet {
    pub layers: Vec<Layer>,
}

#[derive(Clone, Debug)]
pub struct Connection {
    pub from: (usize, usize),
    pub to: (usize, usize),
    pub weight: f32,
}

impl From<ConnectionPacked> for Connection {
    fn from(conn_packed: ConnectionPacked) -> Connection {
        Connection {
            from: conn_packed.get_from(),
            to: conn_packed.get_to(),
            weight: conn_packed.weight as f32 / 1000.0,
        }
    }
}

pub struct ConnectionPacked {
    pub from: u8,
    pub to: u8,
    pub weight: i16,
}

impl ConnectionPacked {
    pub fn to_hex(&self) -> String {
        // weight_bytes[0]
        let mut bytes = vec![];
        let weight_bytes = self.weight.to_le_bytes();
        bytes.push(self.from);
        bytes.push(self.to);
        bytes.push(weight_bytes[0]);
        bytes.push(weight_bytes[1]);
        hex::encode(bytes)
    }

    pub fn get_from(&self) -> (usize, usize) {
        ((self.from >> 4) as usize, (self.from & 0x0F) as usize)
    }

    pub fn get_to(&self) -> (usize, usize) {
        ((self.to >> 4) as usize, (self.to & 0x0F) as usize)
    }

    fn from_hex(hex_string: &str) -> Self {
        let bytes = hex::decode(hex_string).expect("Failed to decode hex");
        // Unpack the values from bytes
        let from = bytes[0];
        let to = bytes[1];
        let weight = i16::from_le_bytes([bytes[2], bytes[3]]);

        ConnectionPacked { from, to, weight }
    }
}

impl From<Connection> for ConnectionPacked {
    fn from(conn: Connection) -> ConnectionPacked {
        ConnectionPacked {
            // Packing x and y coordinates into a single u8
            from: ((conn.from.0 as u8) << 4) | (conn.from.1 as u8 & 0x0F),
            to: ((conn.to.0 as u8) << 4) | (conn.to.1 as u8 & 0x0F),
            weight: (conn.weight * 1000.0) as i16, // Convert weight to i16
        }
    }
}

impl NeuralNet {
    pub fn new(definition: Vec<usize>) -> Self {
        if definition.len() < 2 {
            panic!("Neural network must have at least 2 layers")
        }
        let layers = definition.into_iter().map(Layer::new).collect();

        Self { layers }
    }

    pub fn get_summary_connections(&self) -> Vec<ConnectionPacked> {
        let mut direct_connections: Vec<Connection> = vec![];
        let last_layer_index = self.layers.len() - 1;

        for neuron in self.layers.first().unwrap().neurons.clone() {
            for connection in neuron.connections {
                let (layer_to, _) = connection.to;
                if layer_to == last_layer_index {
                    direct_connections.push(connection)
                }
            }
        }

        direct_connections
            .into_iter()
            .map(|connection| connection.into())
            .collect()
    }

    pub fn for_each_neuron<F>(&self, mut f: F)
    where
        F: FnMut(usize, usize, &Neuron),
    {
        self.layers
            .iter()
            .enumerate()
            .for_each(|(layer_index, layer)| {
                layer
                    .neurons
                    .iter()
                    .enumerate()
                    .for_each(|(neuron_index, neuron)| {
                        f(layer_index, neuron_index, neuron);
                    });
            });
    }

    pub fn tanh(x: f32) -> f32 {
        (x.exp() - (-x).exp()) / (x.exp() + (-x).exp())
    }

    pub fn forward_pass(&mut self, input: Vec<f32>) -> Vec<f32> {
        let mut accumulator: HashMap<(usize, usize), f32> = HashMap::new();
        self.for_each_neuron(|layer_index, neuron_index, neuron| {
            let id = (layer_index, neuron_index);
            let mut input_value = *accumulator.entry(id).or_insert(0.0);
            for connection in &neuron.connections {
                //if if on the input layer use input values as current sum
                let to_id = connection.to;
                if layer_index == 0 {
                    input_value = input[id.1];
                    accumulator.insert(id, input_value);
                }
                let current_output = accumulator.entry(to_id).or_insert(0.0);
                *current_output += input_value * connection.weight;
            }

            //once we've reached a node we can assume we've collated all it's entry connections
            if layer_index != 0 {
                let current_output = accumulator.get_mut(&id).unwrap();
                *current_output = NeuralNet::tanh(*current_output);
            }
        });

        let output_layer_index = self.layers.len() - 1;
        let output_layer_size = self.layers[output_layer_index].neurons.len();

        //return the output layer (tanh)
        return (0..output_layer_size)
            .map(|x| {
                let id = (output_layer_index, x);
                let output = accumulator.get(&id).unwrap();
                NeuralNet::tanh(*output)
            })
            .collect();
    }

    pub fn max_connections(&self) -> usize {
        self.layers.iter().fold(0, |acc, layer| {
            acc + layer.neurons.len() * layer.neurons.len()
        })
    }

    pub fn init_random_connections(&mut self, num_connections: usize, weight_range: (f32, f32)) {
        let mut rng = rand::thread_rng();
        let mut initialized_connections: HashSet<(usize, usize, usize, usize)> = HashSet::new();
        let max_connections = self.max_connections();

        if num_connections > max_connections {
            panic!("Number of connections must be less than the maximum possible connections")
        }

        while initialized_connections.len() < num_connections {
            let layer_from = rng.gen_range(0..(self.layers.len() - 1));
            let neuron_from = rng.gen_range(0..self.layers[layer_from].neurons.len());

            let layer_to = rng.gen_range((layer_from + 1)..self.layers.len());
            let neuron_to = rng.gen_range(0..self.layers[layer_to].neurons.len());

            if initialized_connections.contains(&(layer_from, neuron_from, layer_to, neuron_to)) {
                continue;
            }

            let connection_weight = rng.gen_range(weight_range.0..weight_range.1);
            self.add_connection(
                (layer_from, neuron_from),
                (layer_to, neuron_to),
                connection_weight,
            );
            initialized_connections.insert((layer_from, neuron_from, layer_to, neuron_to));
        }
    }

    pub fn add_connection(&mut self, from: (usize, usize), to: (usize, usize), weight: f32) {
        if from.0 < self.layers.len()
            && to.0 < self.layers.len()
            && from.1 < self.layers[from.0].neurons.len()
            && to.1 < self.layers[to.0].neurons.len()
        {
            self.layers[from.0].neurons[from.1]
                .connections
                .push(Connection { from, to, weight });
        }
    }
}

#[derive(Clone)]
pub struct Layer {
    pub neurons: Vec<Neuron>,
}

impl Layer {
    fn new(length: usize) -> Self {
        if length == 0 {
            panic!("Layer must have at least 1 neuron")
        }
        let neurons = (0..length)
            .map(|_| Neuron {
                connections: Vec::new(),
            })
            .collect();
        Layer { neurons }
    }
}

#[derive(Clone)]
pub struct Neuron {
    //first index is the layer and the second index in the neuron
    pub connections: Vec<Connection>,
}

mod tests {
    use super::*;

    #[test]
    fn test_hex_conversion() {
        let connection = Connection {
            from: (1, 2),
            to: (3, 4),
            weight: 4.32,
        };

        let packed: ConnectionPacked = connection.clone().into();
        let hex = packed.to_hex();
        let repacked = ConnectionPacked::from_hex(&hex);
        let unpacked = Connection::from(repacked);
        assert!(connection.from == unpacked.from);
        assert!(connection.to == unpacked.to);
        assert!(connection.weight == unpacked.weight);
    }

    #[test]
    fn forward_pass() {
        let mut test_net = NeuralNet::new(vec![2, 3, 2]);
        test_net.add_connection((0, 0), (1, 1), 1.2);
        test_net.add_connection((0, 1), (1, 1), 2.1);
        test_net.add_connection((1, 1), (2, 0), 1.0);
        let output = test_net.forward_pass(vec![0.1, 0.2]);
        assert!(output == vec![0.45658463, 0.0]);
    }
}

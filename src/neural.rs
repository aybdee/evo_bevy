use bevy::reflect::Map;
use rand::Rng;
use std::collections::HashMap;

pub struct NeuralNet {
    pub layers: Vec<Layer>,
}

#[derive(Clone)]
pub struct Connection {
    pub to: (usize, usize),
    pub weight: f32,
}

impl NeuralNet {
    pub fn new(definition: Vec<usize>) -> Self {
        if definition.len() < 2 {
            panic!("Neural network must have at least 2 layers")
        }
        let layers = definition.into_iter().map(Layer::new).collect();

        Self { layers }
    }

    fn for_each_neuron<F>(&self, mut f: F)
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

    pub fn init_random_connections(&mut self, num_connections: usize) {
        let mut rng = rand::thread_rng();
        for connection in 0..num_connections {
            let layer_from = rng.gen_range(0..(self.layers.len() - 1));
            let neuron_from = rng.gen_range(0..self.layers[layer_from].neurons.len());

            let layer_to = rng.gen_range((layer_from + 1)..self.layers.len());
            let neuron_to = rng.gen_range(0..self.layers[layer_to].neurons.len());

            //generate  random number between 0 and 1
            let connection_weight = rng.gen_range(0.0..1.0);

            self.add_connection(
                (layer_from, neuron_from),
                (layer_to, neuron_to),
                connection_weight,
            );
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
                .push(Connection { to, weight });
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
    fn forward_pass() {
        let mut test_net = NeuralNet::new(vec![2, 3, 2]);
        test_net.add_connection((0, 0), (1, 1), 1.2);
        test_net.add_connection((0, 1), (1, 1), 2.1);
        test_net.add_connection((1, 1), (2, 0), 1.0);
        let output = test_net.forward_pass(vec![0.1, 0.2]);
        assert!(output == vec![0.45658463, 0.0]);
    }
}

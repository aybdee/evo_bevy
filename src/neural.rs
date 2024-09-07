use rand::Rng;
pub struct NeuralGraph {
    pub layers: Vec<Layer>,
}

#[derive(Clone)]
pub struct Edge {
    pub to: (usize, usize),
    pub weight: f64,
}

impl NeuralGraph {
    pub fn new(definition: Vec<usize>) -> Self {
        if (definition.len() < 2) {
            panic!("Neural network must have at least 2 layers")
        }
        let layers = definition.into_iter().map(Layer::new).collect();

        Self { layers }
    }

    pub fn init_random_connections(&mut self, num_connections: usize) {
        let mut rng = rand::thread_rng();
        for connection in 0..num_connections {
            let layer_from = rng.gen_range(0..(self.layers.len() - 1));
            let neuron_from = rng.gen_range(0..self.layers[layer_from].neurons.len());

            let layer_to = rng.gen_range((layer_from + 1)..self.layers.len());
            let neuron_to = rng.gen_range(0..self.layers[layer_to].neurons.len());

            self.add_connection((layer_from, neuron_from), (layer_to, neuron_to), 1.0);
        }
    }

    pub fn add_connection(&mut self, from: (usize, usize), to: (usize, usize), weight: f64) {
        if from.0 < self.layers.len()
            && to.0 < self.layers.len()
            && from.1 < self.layers[from.0].neurons.len()
            && to.1 < self.layers[to.0].neurons.len()
        {
            self.layers[from.0].neurons[from.1].edges.push(Edge {
                to: to,
                weight: weight,
            });
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
                weight: 0.0,
                edges: Vec::new(),
            })
            .collect();
        Layer { neurons }
    }
}

#[derive(Clone)]
pub struct Neuron {
    pub weight: f64,
    //first index is the layer and the second index in the neuron
    pub edges: Vec<Edge>,
}

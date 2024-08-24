struct NeuralGraph {
    layers: Vec<Layer>,
}

impl NeuralGraph {
    pub fn new(definition: Vec<usize>) -> Self {
        let layers = definition.into_iter().map(Layer::new).collect();

        Self { layers }
    }

    pub fn add_connection(&mut self, from: (usize, usize), to: (usize, usize)) {
        if from.0 < self.layers.len()
            && to.0 < self.layers.len()
            && from.1 < self.layers[from.0].neurons.len()
            && to.1 < self.layers[to.0].neurons.len()
        {
            self.layers[from.0].neurons[from.1].connected.push(to);
        }
    }
}

struct Layer {
    neurons: Vec<Neuron>,
}

impl Layer {
    fn new(length: usize) -> Self {
        let neurons = (0..length)
            .map(|_| Neuron {
                weight: 0.0,
                connected: Vec::new(),
            })
            .collect();
        Layer { neurons }
    }
}

//todo! implement self connections
struct Neuron {
    weight: f64,
    //first index is the layer and the seocond index in the neuron
    connected: Vec<(usize, usize)>,
}

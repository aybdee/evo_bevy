use crate::neural::{self, NeuralGraph, Neuron};

struct Graph {
    pub layers: Vec<Layer>,
}

#[derive(Clone)]
pub struct Layer {
    pub vertices: Vec<Vertex>,
    pub vertex_order: Vec<usize>,
}

impl Layer {
    fn new(length: usize) -> Self {
        let vertices = (0..length)
            .map(|_| Vertex {
                edges: Vec::new(),
                is_dummy: false,
            })
            .collect();
        Layer {
            vertices,
            vertex_order: (0..length).collect(),
        }
    }
}

impl Graph {
    pub fn get_layer(&self, index: usize) -> Option<Layer> {
        if (index >= self.layers.len()) {
            return None;
        } else {
            Some(self.layers[index].clone())
        }
    }

    pub fn get_vertex(&self, index: (usize, usize)) -> Option<(usize, Vertex)> {
        if index.0 >= self.layers.len() {
            return None;
        }

        if index.1 >= self.layers[index.0].vertices.len() {
            return None;
        }

        Some((
            self.layers[index.0].vertex_order[index.1],
            self.layers[index.0].vertices[index.1].clone(),
        ))
    }

    pub fn get_parents(&self, index: (usize, usize)) -> Vec<(usize, usize)> {
        //because there are no long connections we can just assume all the parents are in the
        //immediately previous layer
        if index.0 == 0 {
            panic!("cannot get parents of a vertex in the first layer");
        }
        let mut parents: Vec<(usize, usize)> = Vec::new();
        let parent_layer = index.0 - 1;
        for (vertex_index, vertex) in self
            .get_layer(parent_layer)
            .unwrap()
            .vertices
            .iter()
            .enumerate()
        {
            for edge in &vertex.edges {
                if edge.to == index {
                    parents.push((parent_layer, vertex_index));
                }
            }
        }

        parents
    }

    //get sorted edges for drawing(minimize crossings using the mean heuristic)
    pub fn sort_edges(&mut self) {
        for (layer_index, layer) in self.layers.clone().iter().enumerate() {
            if layer_index == 0 {
                continue;
            } else {
                let sorted_vertices: Vec<usize> = layer
                    .vertices
                    .iter()
                    .enumerate()
                    .map(|(vertex_index, _)| {
                        let parent_positions: Vec<usize> = self
                            .get_parents((layer_index, vertex_index))
                            .iter()
                            .map(|index| self.get_vertex(*index).unwrap().0)
                            .collect();

                        //if item has no parents keep current vertex order
                        if parent_positions.is_empty() {
                            // println!("node ({}, {})", layer_index, vertex_index);
                            layer.vertex_order[vertex_index]
                        } else {
                            let average_position =
                                parent_positions.iter().sum::<usize>() / parent_positions.len();
                            average_position
                        }
                    })
                    .collect();
                self.layers[layer_index].vertex_order = sorted_vertices;
            }
        }
    }
}

impl From<NeuralGraph> for Graph {
    fn from(graph: NeuralGraph) -> Self {
        let layers = graph.layers.clone();
        let mut long_edges: Vec<((usize, usize), neural::Edge)> = Vec::new();
        let mut dummy_neurons: Vec<(usize, usize)> = Vec::new();

        let mut layers: Vec<_> = layers
            .into_iter()
            .enumerate()
            .map(|(layer_index, layer)| {
                let neurons: Vec<_> = layer
                    .neurons
                    .into_iter()
                    .enumerate()
                    .map(|(neuron_index, mut neuron)| {
                        neuron.edges.retain(|edge| {
                            let is_short_edge = edge.to.0 <= layer_index + 1;
                            if !is_short_edge {
                                long_edges.push(((layer_index, neuron_index), edge.clone()));
                            }
                            is_short_edge
                        });
                        neuron
                    })
                    .collect();
                neural::Layer { neurons }
            })
            .collect();

        //add dummy vertices for all the long connections
        for ((from_layer_index, from_neuron_index), edge) in long_edges {
            for layer in (from_layer_index + 1)..edge.to.0 {
                if layer == edge.to.0 - 1 {
                    //connect to child neuron
                    layers[layer].neurons.push(Neuron {
                        weight: 0.0,
                        edges: vec![neural::Edge {
                            to: (edge.to.0, edge.to.1),
                            weight: edge.weight,
                        }],
                    });
                } else {
                    let next_vertex_index = layers[layer + 1].neurons.len();
                    layers[layer].neurons.push(Neuron {
                        weight: 0.0,
                        edges: vec![neural::Edge {
                            to: (layer + 1, next_vertex_index),
                            weight: 0.0,
                        }],
                    });
                }

                let new_neuron_index = layers[layer].neurons.len() - 1;
                if layer == from_layer_index + 1 {
                    //connect to parent neuron
                    layers[from_layer_index].neurons[from_neuron_index]
                        .edges
                        .push(neural::Edge {
                            to: (layer, new_neuron_index),
                            weight: edge.weight,
                        })
                }
                dummy_neurons.push((layer, new_neuron_index));
            }
        }

        let mut new_graph = Graph {
            layers: layers
                .iter()
                .map(|layer| Layer {
                    vertex_order: (0..layer.neurons.len()).collect(),
                    vertices: layer
                        .neurons
                        .iter()
                        .map(|neuron| Vertex {
                            edges: neuron
                                .edges
                                .iter()
                                .map(|edge| Edge {
                                    weight: edge.weight,
                                    to: edge.to,
                                })
                                .collect(),

                            is_dummy: false,
                        })
                        .collect(),
                })
                .collect(),
        };

        //turn all dummy_neurons to dummy_vertices
        for (layer_index, neuron_index) in dummy_neurons {
            new_graph.layers[layer_index].vertices[neuron_index].is_dummy = true;
        }

        new_graph
    }
}

#[derive(Clone)]
pub struct Edge {
    weight: f64,
    to: (usize, usize),
}

#[derive(Clone)]
pub struct Vertex {
    //first index is the layer and the second index is the connected vertex index
    edges: Vec<Edge>,
    is_dummy: bool,
}

mod tests {
    use super::*;

    #[test]
    fn test_graph_conversion() {
        let mut test_net = NeuralGraph::new(vec![2, 3, 2]);
        test_net.add_connection((0, 0), (2, 0), 1.0);
        test_net.add_connection((0, 1), (2, 0), 1.0);
        test_net.add_connection((0, 1), (1, 1), 1.0);
        let test_graph = Graph::from(test_net);
        assert!(test_graph.layers.len() == 3);
        assert!(test_graph.layers[1].vertices.len() == 5);

        assert!(test_graph.layers[1].vertices[3].is_dummy);
        assert!(test_graph.layers[1].vertices[4].is_dummy);
    }

    #[test]
    fn test_vertex_crossing() {
        let mut test_net = NeuralGraph::new(vec![2, 3, 2]);
        test_net.add_connection((0, 0), (2, 0), 1.0);
        test_net.add_connection((0, 1), (2, 0), 1.0);
        test_net.add_connection((0, 1), (1, 1), 1.0);
        let mut test_graph = Graph::from(test_net);
        test_graph.sort_edges();
        assert!(test_graph.layers[1].vertex_order == vec![0, 1, 2, 0, 1]);
        assert!(test_graph.layers[2].vertex_order == vec![0, 1]);
    }
}

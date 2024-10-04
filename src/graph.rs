use crate::neural::{self, NeuralGraph, Neuron};
use bevy::prelude::*;
use bevy_prototype_lyon::prelude::*;
use std::collections::HashMap;

#[derive(Component, Debug)]
pub struct Graph {
    pub layers: Vec<Layer>,
}

#[derive(Debug)]
pub struct DiagramConfig {
    pub position: (f32, f32),
    pub padding: f32,
    pub node_radius: f32,
    pub same_rank_scale: f32,
    pub width: f32,
    pub height: f32,
    pub arrow_thickness: f32,
}

#[derive(Debug)]
pub struct GraphDiagram {
    nodes: Vec<(f32, f32)>,
    edges: Vec<DiagramEdge>,
    config: DiagramConfig,
}
#[derive(Debug)]
pub enum DiagramEdge {
    Straight(StraightEdge),
    Bezier(BezierEdge),
}

#[derive(Debug)]
pub struct StraightEdge {
    pub start: (f32, f32),
    pub end: (f32, f32),
}

#[derive(Debug)]
pub struct BezierEdge {
    pub start: (f32, f32),
    pub end: (f32, f32),
    pub control: (f32, f32),
}

impl GraphDiagram {
    pub fn spawn(self, commands: &mut Commands) {
        commands
            .spawn(SpatialBundle {
                transform: Transform::from_translation(Vec3::new(
                    self.config.position.0,
                    self.config.position.1,
                    0.0,
                )),
                ..Default::default()
            })
            .with_children(|parent| {
                for edge in self.edges {
                    match edge {
                        DiagramEdge::Straight(straight_edge) => {
                            parent.spawn((
                                ShapeBundle {
                                    path: GeometryBuilder::build_as(&shapes::Line(
                                        Vec2::new(straight_edge.start.0, straight_edge.start.1),
                                        Vec2::new(straight_edge.end.0, straight_edge.end.1),
                                    )),
                                    ..default()
                                },
                                Stroke::new(Color::BLACK, self.config.arrow_thickness),
                            ));
                        }
                        _ => {
                            todo!();
                        }
                    }
                }
                for (x, y) in self.nodes {
                    parent.spawn(ShapeBundle {
                        path: GeometryBuilder::build_as(&shapes::Circle {
                            radius: self.config.node_radius,
                            ..shapes::Circle::default()
                        }),
                        spatial: SpatialBundle {
                            transform: Transform::from_translation(Vec3::new(x, y, 0.0)),
                            ..Default::default()
                        },
                        ..default()
                    });
                }
            });
    }
}

#[derive(Clone, Debug)]
pub struct Layer {
    pub vertices: Vec<Vertex>,
    vertex_order: Vec<usize>,
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
    fn for_each_vertex<F>(&self, mut f: F)
    where
        F: FnMut(usize, usize, &Vertex),
    {
        self.layers
            .iter()
            .enumerate()
            .for_each(|(layer_index, layer)| {
                layer
                    .vertices
                    .iter()
                    .enumerate()
                    .for_each(|(vertex_index, vertex)| {
                        f(layer_index, vertex_index, vertex);
                    });
            });
    }

    pub fn get_layer(&self, index: usize) -> Option<Layer> {
        if index >= self.layers.len() {
            None
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

    pub fn get_node_parents(&self, index: (usize, usize)) -> Vec<(usize, usize)> {
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
                            .get_node_parents((layer_index, vertex_index))
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

    pub fn get_diagram(&self, config: DiagramConfig) -> GraphDiagram {
        let view_x_center = config.width / 2.0;
        let num_layers = self.layers.len();

        // Adjust width and height for padding
        let adjusted_width = config.width - (config.padding * 2.0);
        let adjusted_height = config.height - (config.padding * 2.0);
        let mut edges: Vec<DiagramEdge> = vec![];

        // Calculate the width for each layer
        let layer_widths: Vec<f32> = self
            .layers
            .iter()
            .map(|layer| {
                let unique_count = layer
                    .vertex_order
                    .iter()
                    .collect::<std::collections::HashSet<_>>()
                    .len();
                let repeat_count = layer.vertex_order.len() - unique_count;
                unique_count as f32 + (repeat_count as f32 * config.same_rank_scale)
            })
            .collect();

        let max_layer_width = layer_widths.iter().cloned().reduce(f32::max).unwrap();
        let node_spacing = adjusted_width / max_layer_width;
        let layer_spacing = adjusted_height / num_layers as f32;
        let same_pos_spacing = node_spacing / (1.0 / config.same_rank_scale);

        // let mut node_positions: Vec<(f32, f32)> = vec![];
        let mut nodes_with_positions: HashMap<(usize, usize), (f32, f32)> = HashMap::new();

        // Iterate over each layer and calculate node positions
        for (layer_index, layer) in self.layers.iter().enumerate() {
            let layer_y = layer_spacing * layer_index as f32;

            // Sort nodes by vertex order
            let mut nodes_with_order: Vec<_> = layer.vertex_order.iter().enumerate().collect();

            nodes_with_order.sort_by_key(|(_, order)| *order);

            let mut layer_nodes: Vec<(f32, f32)> = vec![];
            let mut last_order = None;
            let mut last_x = 0.0;

            let layer_width = layer_widths[layer_index];
            let to_center_offset = view_x_center - (layer_width * node_spacing) / 2.0;

            // Calculate positions for nodes within the layer
            for (pos, order) in nodes_with_order {
                let x = if let Some(last) = last_order {
                    if last == *order {
                        last_x + same_pos_spacing
                    } else {
                        last_x + node_spacing
                    }
                } else {
                    0.0
                };

                nodes_with_positions.insert((layer_index, pos), (to_center_offset + x, layer_y));
                layer_nodes.push((x, layer_y));
                last_order = Some(*order);
                last_x = x;
            }
        }

        self.for_each_vertex(|layer_index, vertex_index, vertex| {
            let (x, y) = nodes_with_positions
                .get(&(layer_index, vertex_index))
                .unwrap();
            for edge in &vertex.edges {
                let (to_x, to_y) = nodes_with_positions.get(&edge.to).unwrap();
                edges.push(DiagramEdge::Straight(StraightEdge {
                    start: (*x, *y),
                    end: (*to_x, *to_y),
                }));
            }
        });

        GraphDiagram {
            nodes: nodes_with_positions.values().cloned().collect(),
            edges,
            config,
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

#[derive(Clone, Debug)]
pub struct Edge {
    weight: f64,
    to: (usize, usize),
}

#[derive(Clone, Debug)]
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
        println!("{:?}", test_graph);
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

        println!("{:#?}", test_graph);
        test_graph.sort_edges();

        println!("{:#?}", test_graph);
        assert!(test_graph.layers[1].vertex_order == vec![0, 1, 2, 0, 1]);
        assert!(test_graph.layers[2].vertex_order == vec![0, 1]);
    }

    #[test]
    fn test_convert_to_graph() {
        let mut test_net = NeuralGraph::new(vec![2, 3, 2]);
        test_net.add_connection((0, 0), (2, 0), 1.0);
        test_net.add_connection((0, 1), (2, 0), 1.0);
        test_net.add_connection((0, 1), (1, 1), 1.0);
        let mut test_graph = Graph::from(test_net);
        test_graph.sort_edges();
        let diagram = test_graph.get_diagram(DiagramConfig {
            padding: 1.0,
            width: 10.0,
            height: 10.0,
            node_radius: 10.0,
            position: (0.0, 0.0),
            same_rank_scale: 0.5,
            arrow_thickness: 2.0,
        });
        println!("{:?}", diagram);
    }
}

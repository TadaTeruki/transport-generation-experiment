use crate::Site2D;
use terrain_graph::undirected::UndirectedGraph;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct TransportNetwork {
    nodes: Vec<Site2D>,
    graph: UndirectedGraph,
}

#[wasm_bindgen]
pub struct TransportNetworkBuilder {
    start: Site2D,
    iterations: usize,
}

#[wasm_bindgen]
impl TransportNetworkBuilder {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            start: Site2D { x: 0.0, y: 0.0 },
            iterations: 0,
        }
    }

    pub fn set_start(self, start_x: f64, start_y: f64) -> Self {
        Self {
            start: Site2D {
                x: start_x,
                y: start_y,
            },
            ..self
        }
    }

    pub fn set_iterations(self, iterations: usize) -> Self {
        Self { iterations, ..self }
    }

    pub fn build(self) -> TransportNetwork {
        let mut graph = UndirectedGraph::new(2);
        graph.add_edge(0, 1);

        TransportNetwork {
            nodes: vec![
                Site2D {
                    x: self.start.x,
                    y: self.start.y,
                },
                Site2D {
                    x: self.start.x * 0.5,
                    y: self.start.y * 0.5,
                },
            ],
            graph,
        }
    }
}

#[wasm_bindgen]
impl TransportNetwork {
    pub fn num_nodes(&self) -> usize {
        self.nodes.len()
    }

    pub fn get_site(&self, index: usize) -> Site2D {
        self.nodes[index]
    }

    pub fn get_neighbors(&self, index: usize) -> Vec<usize> {
        self.graph.neighbors_of(index).to_vec()
    }
}

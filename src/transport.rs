use std::collections::BinaryHeap;

use rand::{rngs::StdRng, Rng, SeedableRng};
use terrain_graph::undirected::UndirectedGraph;
use wasm_bindgen::prelude::*;

use crate::{terrain::Terrain, Site2D};

static SEA_LEVEL: f64 = 1e-3;

#[wasm_bindgen]
pub struct TransportNetwork {
    nodes: Vec<Site2D>,
    graph: UndirectedGraph,
}

#[wasm_bindgen]
pub struct TransportNetworkBuilder {
    start: Site2D,
    branch_length: f64,
    branch_angle_deviation: f64,
    branch_max_angle: f64,
    rotation_probability: f64,
    iterations: usize,
}
struct Path {
    start: usize,
    end: usize,
    angle: f64,
    cost: f64,
}

impl Ord for Path {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.cost.partial_cmp(&other.cost).unwrap()
    }
}

impl PartialOrd for Path {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.cost.partial_cmp(&other.cost)
    }
}

impl PartialEq for Path {
    fn eq(&self, other: &Self) -> bool {
        self.start == other.start
            && self.end == other.end
            && self.angle == other.angle
            && self.cost == other.cost
    }
}

impl Eq for Path {}

#[wasm_bindgen]
impl TransportNetworkBuilder {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            start: Site2D { x: 0.0, y: 0.0 },
            branch_length: 0.0,
            branch_angle_deviation: 0.0,
            branch_max_angle: 0.0,
            rotation_probability: 0.0,
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

    pub fn set_branch_angle_deviation(self, branch_angle_deviation: f64) -> Self {
        Self {
            branch_angle_deviation,
            ..self
        }
    }

    pub fn set_iterations(self, iterations: usize) -> Self {
        Self { iterations, ..self }
    }

    pub fn set_branch_length(self, branch_length: f64) -> Self {
        Self {
            branch_length,
            ..self
        }
    }

    pub fn set_branch_max_angle(self, branch_max_angle: f64) -> Self {
        Self {
            branch_max_angle,
            ..self
        }
    }

    pub fn set_rotation_probability(self, rotation_probability: f64) -> Self {
        Self {
            rotation_probability,
            ..self
        }
    }

    fn evaluate_cost(
        &self,
        terrain: &Terrain,
        site_from: &Site2D,
        altitude_from: f64,
        site_to: &Site2D,
        altitude_to: f64,
    ) -> Option<f64> {
        if altitude_to < SEA_LEVEL {
            return None;
        }

        let altitude_diff = altitude_to - altitude_from;
        Some(altitude_diff.abs())
    }

    pub fn build(self, seed: u32, terrain: &Terrain) -> TransportNetwork {
        let mut rng = StdRng::seed_from_u64(seed as u64);

        let initial_angle = rng.gen_range(0.0..std::f64::consts::PI);
        let initial_opposite_angle = initial_angle + std::f64::consts::PI;

        let mut sites_collection = vec![
            Site2D {
                x: self.start.x,
                y: self.start.y,
            },
            Site2D {
                x: self.start.x + self.branch_length * initial_angle.cos(),
                y: self.start.y + self.branch_length * initial_angle.sin(),
            },
            Site2D {
                x: self.start.x + self.branch_length * initial_opposite_angle.cos(),
                y: self.start.y + self.branch_length * initial_opposite_angle.sin(),
            },
        ]
        .iter()
        .filter_map(|site| {
            let altitude = terrain.get_altitude(site.x, site.y);
            if let Some(altitude) = altitude {
                Some((*site, altitude))
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

        let mut path_heap = BinaryHeap::new();
        path_heap.push(Path {
            start: 0,
            end: 1,
            angle: initial_angle,
            cost: 0.0,
        });
        path_heap.push(Path {
            start: 0,
            end: 2,
            angle: initial_opposite_angle,
            cost: 0.0,
        });

        let mut final_paths = Vec::new();

        (0..self.iterations).for_each(|_| {
            let current_path = path_heap.pop();
            if current_path.is_none() {
                return;
            }
            let current_path = current_path.unwrap();
            let site_start = sites_collection[current_path.start];
            let site_end = sites_collection[current_path.end];

            let mut site_next: Option<Site2D> = None;
            let mut min_cost = std::f64::MAX;
            let mut min_cost_angle = current_path.angle;
            let mut min_cost_altitude = 0.0;
            let check_times = (self.branch_max_angle / self.branch_angle_deviation).ceil() as usize;

            let rotation_iteration_start = {
                if rng.gen_bool(self.rotation_probability) {
                    -1
                } else {
                    0
                }
            };
            let rotation_iteration_end = {
                if rng.gen_bool(self.rotation_probability) {
                    1
                } else {
                    0
                }
            };

            (rotation_iteration_start..rotation_iteration_end + 1).for_each(|riter| {
                let current_angle = current_path.angle + riter as f64 * std::f64::consts::PI * 0.5;
                (0..check_times).for_each(|i| {
                    let angle = current_angle + self.branch_angle_deviation * (i as f64);
                    let site_a = Site2D {
                        x: site_end.0.x + self.branch_length * angle.cos(),
                        y: site_end.0.y + self.branch_length * angle.sin(),
                    };
                    let altitude_a = terrain.get_altitude(site_a.x, site_a.y);
                    if let Some(altitude_a) = altitude_a {
                        if let Some(cost) = self.evaluate_cost(
                            terrain,
                            &site_start.0,
                            site_start.1,
                            &site_a,
                            altitude_a,
                        ) {
                            if cost < min_cost {
                                min_cost = cost;
                                min_cost_angle = angle;
                                min_cost_altitude = altitude_a;
                                site_next = Some(site_a);
                            }
                        }
                    }

                    if i == 0 {
                        return;
                    }
                    let angle = current_angle - self.branch_angle_deviation * (i as f64);
                    let site_b = Site2D {
                        x: site_end.0.x + self.branch_length * angle.cos(),
                        y: site_end.0.y + self.branch_length * angle.sin(),
                    };
                    let altitude_b = terrain.get_altitude(site_b.x, site_b.y);
                    if let Some(altitude_b) = altitude_b {
                        if let Some(cost) = self.evaluate_cost(
                            terrain,
                            &site_start.0,
                            site_start.1,
                            &site_b,
                            altitude_b,
                        ) {
                            if cost < min_cost {
                                min_cost = cost;
                                min_cost_angle = angle;
                                min_cost_altitude = altitude_b;
                                site_next = Some(site_b);
                            }
                        }
                    }
                });

                if let Some(site_next) = site_next {
                    let site_next_index = sites_collection.len();
                    sites_collection.push((site_next, min_cost_altitude));
                    path_heap.push(Path {
                        start: current_path.end,
                        end: site_next_index,
                        angle: min_cost_angle,
                        cost: min_cost,
                    });
                }
            });

            final_paths.push(current_path);
        });

        let mut graph = UndirectedGraph::new(sites_collection.len());

        final_paths.iter().for_each(|path| {
            graph.add_edge(path.start, path.end);
        });

        TransportNetwork {
            nodes: sites_collection
                .iter()
                .map(|(site, _)| *site)
                .collect::<Vec<_>>(),
            graph,
        }
        /*
        TransportNetwork {
            nodes: Vec::new(),
            graph: UndirectedGraph::new(0),
        }
        */
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

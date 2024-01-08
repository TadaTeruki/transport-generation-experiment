use std::collections::BinaryHeap;

use rand::{rngs::StdRng, Rng, SeedableRng};
use terrain_graph::edge_attributed_undirected::EdgeAttributedUndirectedGraph;
use wasm_bindgen::prelude::*;

use crate::{
    terrain::Terrain,
    transport::{
        math::get_cross,
        treeobj::{PathTree, PathTreeQuery},
    },
    Site2D,
};

static SEA_LEVEL: f64 = 1e-3;

#[derive(Clone, Copy, Default)]
pub(crate) struct PathAttr {
    is_highway: bool,
    is_even: bool,
}

#[wasm_bindgen]
pub struct TransportNetwork {
    nodes: Vec<Site2D>,
    graph: EdgeAttributedUndirectedGraph<PathAttr>,
}

#[wasm_bindgen]
pub struct TransportNetworkBuilder {
    start: Site2D,
    branch_length: f64,
    branch_angle_deviation: f64,
    branch_max_angle: f64,
    highway_rotation_probability: f64,
    normal_rotation_probability: f64,
    highway_construction_priority: f64,
    even_path_length_weight: f64,
    highway_path_length_weight: f64,
    iterations: usize,
}
struct Path {
    start: usize,
    end: usize,
    angle: f64,
    cost: f64,
    path_attr: PathAttr,
}

impl Ord for Path {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other.cost.partial_cmp(&self.cost).unwrap()
    }
}

impl PartialOrd for Path {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        other.cost.partial_cmp(&self.cost)
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
            highway_rotation_probability: 0.0,
            normal_rotation_probability: 0.0,
            iterations: 0,
            highway_construction_priority: 0.0,
            even_path_length_weight: 0.0,
            highway_path_length_weight: 0.0,
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

    pub fn set_highway_rotation_probability(self, highway_rotation_probability: f64) -> Self {
        Self {
            highway_rotation_probability,
            ..self
        }
    }

    pub fn set_normal_rotation_probability(self, normal_rotation_probability: f64) -> Self {
        Self {
            normal_rotation_probability,
            ..self
        }
    }

    pub fn set_highway_construction_priority(self, highway_construction_priority: f64) -> Self {
        Self {
            highway_construction_priority,
            ..self
        }
    }

    pub fn set_even_path_length_weight(self, even_path_length_weight: f64) -> Self {
        Self {
            even_path_length_weight,
            ..self
        }
    }

    pub fn set_highway_path_length_weight(self, highway_path_length_weight: f64) -> Self {
        Self {
            highway_path_length_weight,
            ..self
        }
    }

    fn evaluate_cost(&self, altitude_from: f64, altitude_to: f64, attr: PathAttr) -> Option<f64> {
        if altitude_to < SEA_LEVEL {
            return None;
        }

        let mut altitude_diff = altitude_to - altitude_from;
        if attr.is_even {
            altitude_diff *= self.even_path_length_weight;
        }
        if attr.is_highway {
            altitude_diff *= self.highway_path_length_weight;
        }
        Some(
            altitude_diff.abs()
                * altitude_to
                * (1.0 / self.highway_construction_priority + (!attr.is_highway as i32) as f64),
        )
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
            path_attr: PathAttr {
                is_highway: true,
                is_even: false,
            },
        });
        path_heap.push(Path {
            start: 0,
            end: 2,
            angle: initial_opposite_angle,
            cost: 0.0,
            path_attr: PathAttr {
                is_highway: true,
                is_even: false,
            },
        });

        let mut path_tree = PathTree::new();
        (0..self.iterations).for_each(|_| {
            let current_path = path_heap.pop();
            if current_path.is_none() {
                return;
            }
            let current_path = current_path.unwrap();
            let site_start = sites_collection[current_path.start];
            let site_end = sites_collection[current_path.end];

            let intersection_distance = self.branch_length * 0.8;

            // find path intersection
            let intersection = path_tree.find(
                &site_start.0,
                &site_end.0,
                intersection_distance,
                &[current_path.start],
            );
            let mut intersection_pushed = false;
            if let PathTreeQuery::Site(site_index) = intersection {
                path_tree.insert(
                    current_path.start,
                    site_index,
                    site_start.0,
                    sites_collection[site_index].0,
                    current_path.path_attr,
                );
                intersection_pushed = true;
            } else if let PathTreeQuery::Path(intersection) = intersection {
                let cross = get_cross(
                    intersection.site_start,
                    intersection.site_end,
                    site_start.0,
                    site_end.0,
                );
                if let Some(cross) = cross {
                    intersection_pushed = true;
                    if cross.1 {
                        let cross_site = cross.0;
                        let altitude = terrain.get_altitude(cross_site.x, cross_site.y);
                        if let Some(altitude) = altitude {
                            // push
                            let site_next_index = sites_collection.len();
                            sites_collection.push((cross_site, altitude));
                            path_tree.split(*intersection, &cross_site, site_next_index);
                            path_tree.insert(
                                current_path.start,
                                site_next_index,
                                site_start.0,
                                cross_site,
                                current_path.path_attr,
                            );
                        }
                    }
                }
            }

            if intersection_pushed {
                return;
            }
            path_tree.insert(
                current_path.start,
                current_path.end,
                site_start.0,
                site_end.0,
                current_path.path_attr,
            );

            let check_times =
                (self.branch_max_angle / self.branch_angle_deviation).floor() as usize;

            (-1..2).for_each(|riter| {
                let mut site_next: Option<Site2D> = None;
                let mut min_cost = std::f64::MAX;
                let mut min_cost_angle = 0.0;
                let mut min_cost_altitude = 0.0;

                let mut is_highway = current_path.path_attr.is_highway;
                let mut is_even = current_path.path_attr.is_even;
                if riter != 0 {
                    is_even = !is_even;
                    is_highway = false;
                    if current_path.path_attr.is_highway
                        && rng.gen_bool(self.highway_rotation_probability)
                    {
                        is_highway = true;
                    } else if !rng.gen_bool(self.normal_rotation_probability) {
                        return;
                    }
                }
                let site_next_attr = PathAttr {
                    is_highway,
                    is_even,
                };

                let current_angle = current_path.angle + riter as f64 * std::f64::consts::PI * 0.5;
                (0..check_times + 1).for_each(|i| {
                    let branch_length = {
                        let mut branch_length = self.branch_length;
                        if site_next_attr.is_even {
                            branch_length *= self.even_path_length_weight
                        }
                        if site_next_attr.is_highway {
                            branch_length *= self.highway_path_length_weight
                        }
                        branch_length
                    };
                    let angle = current_angle + self.branch_angle_deviation * (i as f64);
                    let site_a = Site2D {
                        x: site_end.0.x + branch_length * angle.cos(),
                        y: site_end.0.y + branch_length * angle.sin(),
                    };
                    let altitude_a = terrain.get_altitude(site_a.x, site_a.y);
                    if let Some(altitude_a) = altitude_a {
                        if let Some(cost) =
                            self.evaluate_cost(site_start.1, altitude_a, site_next_attr)
                        {
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
                        x: site_end.0.x + branch_length * angle.cos(),
                        y: site_end.0.y + branch_length * angle.sin(),
                    };
                    let altitude_b = terrain.get_altitude(site_b.x, site_b.y);
                    if let Some(altitude_b) = altitude_b {
                        if let Some(cost) =
                            self.evaluate_cost(site_start.1, altitude_b, site_next_attr)
                        {
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
                        path_attr: site_next_attr,
                    });
                }
            });
        });

        let mut graph = EdgeAttributedUndirectedGraph::new(sites_collection.len());

        path_tree.for_each(|path| {
            if graph.has_edge(path.site_index_start, path.site_index_end).0 {
                return;
            }
            graph.add_edge(path.site_index_start, path.site_index_end, path.path_attr);
        });

        TransportNetwork {
            nodes: sites_collection
                .iter()
                .map(|(site, _)| *site)
                .collect::<Vec<_>>(),
            graph,
        }
    }
}

#[wasm_bindgen]
pub struct Neighbor {
    pub index: usize,
    pub is_highway: bool,
}

#[wasm_bindgen]
impl TransportNetwork {
    pub fn num_nodes(&self) -> usize {
        self.nodes.len()
    }

    pub fn get_site(&self, index: usize) -> Site2D {
        self.nodes[index]
    }

    pub fn get_neighbors(&self, index: usize) -> Vec<Neighbor> {
        self.graph
            .neighbors_of(index)
            .iter()
            .map(|n| Neighbor {
                index: n.0,
                is_highway: n.1.is_highway,
            })
            .collect::<Vec<_>>()
    }
}

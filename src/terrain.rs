use fastlem::core::{parameters::TopographicalParameters, traits::Model};
use fastlem::lem::generator::TerrainGenerator;
use fastlem::models::surface::builder::TerrainModel2DBulider;
use fastlem::models::surface::terrain::Terrain2D;
use noise::{NoiseFn, Perlin};
use wasm_bindgen::prelude::*;

use crate::Site2D;

fn octaved_perlin(perlin: &Perlin, x: f64, y: f64, octaves: usize, persistence: f64) -> f64 {
    let mut value = 0.0;
    let mut amplitude = 1.0;
    let mut frequency = 1.0;
    let mut max_value = 0.0;

    for _ in 0..octaves {
        value += perlin.get([x * frequency, y * frequency]) * amplitude;
        max_value += amplitude;
        amplitude *= persistence;
        frequency *= 2.0;
    }

    value / max_value
}

#[wasm_bindgen]
pub struct TerrainBuilder {
    bound_min: Site2D,
    bound_max: Site2D,
    node_num: usize,
    seed: u32,
}

#[wasm_bindgen]
pub struct Terrain {
    terrain: Terrain2D,
}

#[wasm_bindgen]
impl TerrainBuilder {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            bound_min: Site2D { x: 0.0, y: 0.0 },
            bound_max: Site2D { x: 0.0, y: 0.0 },
            node_num: 0,
            seed: 0,
        }
    }

    pub fn set_bound_min(self, bound_min_x: f64, bound_min_y: f64) -> Self {
        Self {
            bound_min: Site2D {
                x: bound_min_x,
                y: bound_min_y,
            },
            ..self
        }
    }

    pub fn set_bound_max(self, bound_max_x: f64, bound_max_y: f64) -> Self {
        Self {
            bound_max: Site2D {
                x: bound_max_x,
                y: bound_max_y,
            },
            ..self
        }
    }

    pub fn set_node_num(self, node_num: usize) -> Self {
        Self { node_num, ..self }
    }

    pub fn set_seed(self, seed: u32) -> Self {
        Self { seed, ..self }
    }

    pub fn build(&self) -> Terrain {
        let model = TerrainModel2DBulider::from_random_sites(
            self.node_num,
            self.bound_min.into(),
            self.bound_max.into(),
        )
        .relaxate_sites(1)
        .unwrap()
        .build()
        .unwrap();

        let sites = model.sites().to_vec();

        let perlin = Perlin::new(self.seed);

        let terrain = TerrainGenerator::default()
            .set_model(model)
            .set_parameters(
                (0..sites.len())
                    .map(|i| {
                        let site = sites[i];
                        let octaves = 8;
                        let x = site.x / (self.bound_max.x - self.bound_min.x);
                        let y = site.y / (self.bound_max.y - self.bound_min.y);
                        let dist_from_center = ((x - 0.5).powi(2) + (y - 0.5).powi(2)).sqrt();
                        let noise_erodibility =
                            octaved_perlin(&perlin, x * 0.5, y * 0.5, octaves, 0.55)
                                .abs()
                                .powi(2)
                                * 1.0
                                + (1.0 - dist_from_center).powi(2) * 3.0;
                        let noise_is_outlet = (octaved_perlin(&perlin, x, y, octaves, 0.5) * 0.5
                            + 0.5)
                            * dist_from_center
                            + (1.0 - dist_from_center) * 0.5;
                        TopographicalParameters::default()
                            .set_erodibility(noise_erodibility)
                            .set_is_outlet(noise_is_outlet > 0.55)
                    })
                    .collect::<_>(),
            )
            .generate()
            .unwrap();

        Terrain { terrain }
    }
}

#[wasm_bindgen]
impl Terrain {
    pub fn get_altitude(&self, site_x: f64, site_y: f64) -> Option<f64> {
        let site = Site2D {
            x: site_x,
            y: site_y,
        };
        self.terrain.get_altitude(&site.into())
    }
}

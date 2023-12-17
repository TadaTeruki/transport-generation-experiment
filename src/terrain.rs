use fastlem::core::{parameters::TopographicalParameters, traits::Model};
use fastlem::lem::generator::TerrainGenerator;
use fastlem::models::surface::terrain::Terrain2D;
use fastlem::models::surface::{builder::TerrainModel2DBulider, sites::Site2D};
use noise::{NoiseFn, Perlin};

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

pub struct Terrain {
    terrain: Terrain2D,
    bound_min: Site2D,
    bound_max: Site2D,
}

impl Terrain {
    pub fn new(num: usize, bound_min: Site2D, bound_max: Site2D) -> Self {
        let model = TerrainModel2DBulider::from_random_sites(num, bound_min, bound_max)
            .relaxate_sites(1)
            .unwrap()
            .build()
            .unwrap();

        let sites = model.sites().to_vec();

        let perlin = Perlin::new(72);

        let terrain = TerrainGenerator::default()
            .set_model(model)
            .set_parameters(
                (0..sites.len())
                    .map(|i| {
                        let site = sites[i];
                        let octaves = 8;
                        let x = site.x / (bound_max.x - bound_min.x);
                        let y = site.y / (bound_max.y - bound_min.y);
                        let dist_from_center = ((x - 0.5).powi(2) + (y - 0.5).powi(2)).sqrt();
                        let noise_erodibility =
                            octaved_perlin(&perlin, x * 0.5, y * 0.5, octaves, 0.55)
                                .abs()
                                .powi(2)
                                * 15.0
                                + (1.0 - dist_from_center) * 1.8;
                        let noise_is_outlet =
                            octaved_perlin(&perlin, x, y, octaves, 0.5) * 0.5 + 0.5;
                        TopographicalParameters::default()
                            .set_erodibility(noise_erodibility)
                            .set_is_outlet(noise_is_outlet > 0.6)
                    })
                    .collect::<_>(),
            )
            .generate()
            .unwrap();

        Self {
            terrain,
            bound_min,
            bound_max,
        }
    }

    pub fn render_terrain(
        &self,
        img_width: u32,
        img_height: u32,
    ) -> image::ImageBuffer<image::Rgb<u8>, Vec<u8>> {
        // (color: [u8; 3], altitude: f64)
        let color_table: Vec<([u8; 3], f64)> = vec![
            ([70, 150, 200], 0.0),
            ([240, 240, 210], 0.1),
            ([190, 200, 120], 0.3),
            ([25, 100, 25], 6.0),
            ([15, 60, 15], 8.0),
        ];

        // get color from altitude
        let get_color = |altitude: f64| -> [u8; 3] {
            let color_index = {
                let mut i = 0;
                while i < color_table.len() {
                    if altitude < color_table[i].1 {
                        break;
                    }
                    i += 1;
                }
                i
            };

            if color_index == 0 {
                color_table[0].0
            } else if color_index == color_table.len() {
                color_table[color_table.len() - 1].0
            } else {
                let color_a = color_table[color_index - 1];
                let color_b = color_table[color_index];

                let prop_a = color_a.1;
                let prop_b = color_b.1;

                let prop = (altitude - prop_a) / (prop_b - prop_a);

                [
                    (color_a.0[0] as f64 + (color_b.0[0] as f64 - color_a.0[0] as f64) * prop)
                        as u8,
                    (color_a.0[1] as f64 + (color_b.0[1] as f64 - color_a.0[1] as f64) * prop)
                        as u8,
                    (color_a.0[2] as f64 + (color_b.0[2] as f64 - color_a.0[2] as f64) * prop)
                        as u8,
                ]
            }
        };

        let mut image_buf = image::RgbImage::new(img_width, img_height);

        for imgx in 0..img_width {
            for imgy in 0..img_height {
                let x = self.bound_min.x
                    + (self.bound_max.x - self.bound_min.x) * (imgx as f64 / img_width as f64);
                let y = self.bound_min.y
                    + (self.bound_max.y - self.bound_min.y) * (imgy as f64 / img_height as f64);
                let site = Site2D { x, y };
                let altitude = self.terrain.get_altitude(&site);
                if let Some(altitude) = altitude {
                    let color = get_color(altitude);
                    image_buf.put_pixel(imgx, imgy, image::Rgb(color));
                }
            }
        }

        image_buf
    }
}

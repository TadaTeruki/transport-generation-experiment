/*
use fastlem::models::surface::sites::Site2D;

mod terrain;

fn main() {
    let num = 30000;
    let bound_min = Site2D { x: 0.0, y: 0.0 };
    let bound_max = Site2D { x: 100.0, y: 100.0 };

    let terrain = terrain::Terrain::new(num, bound_min, bound_max);

    let img_width = 500;
    let img_height = 500;
    let image_buf = terrain.render_terrain(img_width, img_height);
    image_buf.save("image.png").unwrap();
}
*/

pub mod terrain;
pub mod transport;

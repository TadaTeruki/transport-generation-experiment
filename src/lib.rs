pub mod terrain;
pub mod transport;

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[derive(Clone, Copy)]
pub struct Site2D {
    pub x: f64,
    pub y: f64,
}

impl Into<fastlem::models::surface::sites::Site2D> for Site2D {
    fn into(self) -> fastlem::models::surface::sites::Site2D {
        fastlem::models::surface::sites::Site2D {
            x: self.x,
            y: self.y,
        }
    }
}

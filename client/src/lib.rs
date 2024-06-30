use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct Canvas {
    width: usize,
    height: usize,
    pixels: Vec<String>,
}

#[wasm_bindgen]
impl Canvas {
    #[wasm_bindgen(constructor)]
    pub fn new(width: usize, height: usize) -> Canvas {
        let pixels = vec!["#FFFFFF".to_string(); width * height];
        Canvas { width, height, pixels }
    }

    pub fn update_pixel(&mut self, x: usize, y: usize, color: String) {
        let index = y * self.width + x;
        self.pixels[index] = color;
    }

    pub fn get_pixel(&self, x: usize, y: usize) -> String {
        let index = y * self.width + x;
        self.pixels[index].clone()
    }
}
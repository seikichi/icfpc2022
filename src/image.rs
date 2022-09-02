use crate ::isl;

use image;
use image::GenericImageView;

#[derive(Debug, Clone, PartialEq)]
pub struct Image(pub Vec<Vec<isl::Color>>);

pub fn open(path: &str) -> Image {
    let img = image::open(path).unwrap();
    let (w, h) = img.dimensions();

    let mut result = Image(vec![vec![isl::Color { r: 0.0, g: 0.0, b: 0.0, a: 0.0}; w as usize]; h as usize]);

    for pixel in img.pixels() {
        let x = pixel.0 as usize;
        let y = pixel.1 as usize;
        let [r, g, b, a] = pixel.2.0;
        result.0[y][x].r = r as f32;
        result.0[y][x].g = g as f32;
        result.0[y][x].b = b as f32;
        result.0[y][x].a = a as f32;
    }

    result
}

use crate ::isl;

use image;
use image::GenericImageView;

#[derive(Debug, Clone, PartialEq)]
pub struct Image(pub Vec<Vec<isl::Color>>);

pub fn open(path: &str) -> Image {
    let img = image::open(path).unwrap();
    let (w, h) = img.dimensions();

    let mut result = Image(vec![vec![glam::Vec4::ZERO; w as usize]; h as usize]);

    for pixel in img.pixels() {
        let x = pixel.0 as usize;
        let y = pixel.1 as usize;
        let [r, g, b, a] = pixel.2.0;
        result.0[y][x].x = (r as f32) / 255.0;
        result.0[y][x].y = (g as f32) / 255.0;
        result.0[y][x].z = (b as f32) / 255.0;
        result.0[y][x].w = (a as f32) / 255.0;
    }

    result
}

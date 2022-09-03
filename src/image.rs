use crate::isl;

use image::GenericImageView;
use image::{self, RgbaImage};

#[derive(Debug, Clone, PartialEq)]
pub struct Image(pub Vec<Vec<isl::Color>>);
impl Image {
    pub fn width(&self) -> usize {
        self.0[0].len()
    }
    pub fn height(&self) -> usize {
        self.0.len()
    }
    pub fn area(&self) -> usize {
        self.width() * self.height()
    }
    pub fn new(w: usize, h: usize) -> Self {
        Image(vec![vec![glam::Vec4::ZERO; w as usize]; h as usize])
    }
    pub fn save(&self, path: &str) {
        let mut img = RgbaImage::new(self.width() as u32, self.height() as u32);
        for pixel in img.enumerate_pixels_mut() {
            let x = pixel.0 as usize;
            let y = (self.height() - pixel.1 as usize - 1);
            let r = ((self.0[y][x].x * 255.0).round() as u8);
            let g = ((self.0[y][x].y * 255.0).round() as u8);
            let b = ((self.0[y][x].z * 255.0).round() as u8);
            let a = ((self.0[y][x].w * 255.0).round() as u8);
            *pixel.2 = image::Rgba([r, g, b, a]);
        }
        img.save(path).unwrap();
    }
}

pub fn open(path: &str) -> Image {
    let img = image::open(path).unwrap();
    let (w, h) = img.dimensions();

    let mut result = Image::new(w as usize, h as usize);

    for pixel in img.pixels() {
        let x = pixel.0 as usize;
        let y = (h - pixel.1 - 1) as usize;
        let [r, g, b, a] = pixel.2 .0;
        result.0[y][x].x = (r as f32) / 255.0;
        result.0[y][x].y = (g as f32) / 255.0;
        result.0[y][x].z = (b as f32) / 255.0;
        result.0[y][x].w = (a as f32) / 255.0;
    }

    result
}

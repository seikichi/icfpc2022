use std::fmt::Display;

use crate::isl::{self, Color};

use glam::IVec4;
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
        Image(vec![vec![glam::Vec4::ONE; w as usize]; h as usize])
    }
    #[allow(dead_code)]
    pub fn save(&self, path: &str) {
        let mut img = RgbaImage::new(self.width() as u32, self.height() as u32);
        for pixel in img.enumerate_pixels_mut() {
            let x = pixel.0 as usize;
            let y = self.height() - pixel.1 as usize - 1;
            let r = (self.0[y][x].x * 255.0).round() as u8;
            let g = (self.0[y][x].y * 255.0).round() as u8;
            let b = (self.0[y][x].z * 255.0).round() as u8;
            let a = (self.0[y][x].w * 255.0).round() as u8;
            *pixel.2 = image::Rgba([r, g, b, a]);
        }
        img.save(path).unwrap();
    }
    pub fn from_string_array(string_array: &[&str], pen_color: Color) -> Self {
        let white = Color::ONE;
        let v = string_array
            .into_iter()
            .map(|row| {
                row.chars()
                    .map(|c| if c == '.' { white } else { pen_color })
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();
        Image(v)
    }
}
impl Display for Image {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut buf = String::new();
        for row in self.0.iter() {
            for pixel in row.iter() {
                let c = (*pixel * 255.0).round().as_ivec4();
                if c == IVec4::new(255, 255, 255, 255) {
                    buf.push('.')
                } else if c == IVec4::new(0, 0, 0, 255) {
                    buf.push('#')
                } else if c == IVec4::new(255, 0, 0, 255) {
                    buf.push('r')
                } else if c == IVec4::new(0, 255, 0, 255) {
                    buf.push('g')
                } else if c == IVec4::new(0, 0, 255, 255) {
                    buf.push('b')
                } else if c == IVec4::new(0, 0, 0, 0) {
                    buf.push('z')
                } else {
                    buf.push('x')
                }
            }
            buf.push('\n')
        }
        buf.fmt(f)
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

use std::fmt::Display;

use crate::isl::{self, Color};

use glam::IVec4;
use image::GenericImageView;
use image::{self, RgbaImage};

#[derive(Debug, Clone, PartialEq)]
pub struct Image(pub Vec<Vec<isl::Color>>);
impl Image {
    pub fn new(w: usize, h: usize) -> Self {
        Image(vec![vec![glam::Vec4::ONE; w as usize]; h as usize])
    }
    pub fn width(&self) -> usize {
        self.0[0].len()
    }
    pub fn height(&self) -> usize {
        self.0.len()
    }
    pub fn area(&self) -> usize {
        self.width() * self.height()
    }
    #[allow(dead_code)]
    pub fn average(&self, p: isl::Point, size: isl::Point) -> Color {
        let mut sum = Color::ZERO;
        for y in p.y..(p.y + size.y) {
            for x in p.x..(p.x + size.x) {
                sum += self.0[y as usize][x as usize];
            }
        }
        return sum / (size.y * size.x) as f32;
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
    #[allow(dead_code)]
    pub fn from_string_array(string_array: &[&str]) -> Self {
        let v = string_array
            .into_iter()
            .map(|row| {
                row.chars()
                    .map(|c| match c {
                        '.' => Color::ONE,
                        '#' => Color::new(0.0, 0.0, 0.0, 1.0),
                        'r' => Color::new(1.0, 0.0, 0.0, 1.0),
                        'g' => Color::new(0.0, 1.0, 0.0, 1.0),
                        'b' => Color::new(0.0, 0.0, 1.0, 1.0),
                        'z' => Color::ZERO,
                        _ => Color::new(0.5, 0.5, 0.5, 1.0),
                    })
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

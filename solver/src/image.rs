use std::fmt::Display;
use std::path::Path;

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
    pub fn save<P: AsRef<Path>>(&self, path: P) -> anyhow::Result<()> {
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
        img.save(path)?;
        Ok(())
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

pub fn open<P: AsRef<Path>>(path: P) -> anyhow::Result<Image> {
    let img = image::open(path)?;
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

    Ok(result)
}

// k-means を用いて画像の代表色を求める
pub fn k_means_color_sampling(
    image: &Image,
    n_colors: usize,
    n_iter: usize,
    rng: &mut impl rand::Rng,
) -> Vec<Color> {
    let mut initial_samples: Vec<Color> = vec![];
    'outer: for _i in 0..1000000 {
        // 近い色は避けてサンプルの色を何個か取得する
        let x = rng.gen_range(0..image.width());
        let y = rng.gen_range(0..image.height());
        let c = image.0[y][x];
        for &pc in initial_samples.iter() {
            if ((pc - c) * 255.0).length() < 30.0 {
                continue 'outer;
            }
        }
        initial_samples.push(c);
        if initial_samples.len() == n_colors {
            break;
        }
    }

    // k-means
    let mut samples = initial_samples;
    for _i in 0..n_iter {
        let mut sum = vec![Color::ZERO; samples.len()];
        let mut count = vec![0; samples.len()];

        for row in &image.0 {
            for pixel in row {
                let mut min_diff = 1000000.0;
                let mut best_cluster = 0;
                for (i_cluster, color) in samples.iter().enumerate() {
                    let diff = (*pixel - *color).length();
                    if diff < min_diff {
                        min_diff = diff;
                        best_cluster = i_cluster;
                    }
                }
                sum[best_cluster] += *pixel;
                count[best_cluster] += 1;
            }
        }

        for i in 0..samples.len() {
            samples[i] = sum[i] / (count[i] as f32);
        }
    }

    samples
}

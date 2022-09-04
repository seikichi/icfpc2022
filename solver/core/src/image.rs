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
    pub fn majority(&self, p: isl::Point, size: isl::Point) -> Color {
        // TODO: 高速化（release buildだとまだ早い）
        let mut all_pixels = vec![];
        for y in p.y..(p.y + size.y) {
            for x in p.x..(p.x + size.x) {
                all_pixels.push(self.0[y as usize][x as usize])
            }
        }
        let mut max_occurrence = 0;
        let mut most_occurred_color: Color = Color::ZERO;
        for i in 0..all_pixels.len() {
            let mut count = 0;
            let pix = all_pixels[i];
            for j in i..all_pixels.len() {
                if pix == all_pixels[j] {
                    count += 1;
                }
            }
            if count > max_occurrence {
                max_occurrence = count;
                most_occurred_color = pix.clone();
            }
        }
        return most_occurred_color;
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
    sx: usize,
    sy: usize,
    w: usize,
    h: usize,
    rng: &mut impl rand::Rng,
) -> Vec<Color> {
    // 初期解をいい感じに作る
    assert!(sx + w <= image.width());
    assert!(sy + h <= image.height());
    let mut initial_samples: Vec<Color> = vec![];
    let c = {
        let x = rng.gen_range(sx..(sx + w));
        let y = rng.gen_range(sy..(sy + h));
        image.0[y][x]
    };
    initial_samples.push(c);
    let mut iter = 0;
    while initial_samples.len() < n_colors {
        // 最も近いサンプルまでの二乗距離
        let mut nsd = vec![vec![0.0; w]; h];
        for dy in 0..h {
            for dx in 0..w {
                let pixel = image.0[sy + dy][sx + dx];
                let mut min_diff = 10000000.0;
                for &sc in &initial_samples {
                    let diff = (sc - pixel).length_squared();
                    if diff < min_diff {
                        min_diff = diff;
                    }
                }
                nsd[dy][dx] = min_diff;
            }
        }
        // SD(x) / ∑SD(x) の確率でピクセルを選ぶ
        let d_sum = nsd.iter().map(|row| row.iter().sum::<f32>()).sum::<f32>();
        let p = rng.gen::<f32>();
        let mut cumsum = 0.0;
        'outer: for (dy, row) in nsd.into_iter().enumerate() {
            for (dx, sd) in row.into_iter().enumerate() {
                cumsum += sd / d_sum;
                if p < cumsum {
                    initial_samples.push(image.0[sy + dy][sx + dx]);
                    break 'outer;
                }
            }
        }
        iter += 1;
        if iter == 100 {
            break;
        }
    }

    // k-means
    let mut samples = initial_samples;
    for _i in 0..n_iter {
        let mut sum = vec![Color::ZERO; samples.len()];
        let mut count = vec![0; samples.len()];

        for dy in 0..h {
            for dx in 0..w {
                let pixel = image.0[sy + dy][sx + dx];
                let mut min_diff = 10000000.0;
                let mut best_cluster = 0;
                for (i_cluster, &color) in samples.iter().enumerate() {
                    let diff = (pixel - color).length_squared();
                    if diff < min_diff {
                        min_diff = diff;
                        best_cluster = i_cluster;
                    }
                }
                sum[best_cluster] += pixel;
                count[best_cluster] += 1;
            }
        }

        for i in 0..samples.len() {
            samples[i] = sum[i] / (count[i] as f32);
        }
    }

    samples
}

// image の各ピクセルを samples の中で一番近い色に破壊的に置き換える
#[allow(dead_code)]
pub fn replace_pixels_to_nearest_samples(image: &mut Image, samples: &[Color]) {
    for row in image.0.iter_mut() {
        for pixel in row.iter_mut() {
            let mut min_diff = 10000000.0;
            let mut best_color = Color::ZERO;
            for color in samples {
                let diff = (*pixel - *color).length_squared();
                if diff < min_diff {
                    min_diff = diff;
                    best_color = *color;
                }
            }
            *pixel = best_color;
        }
    }
}

/*
#[cfg(test)]
mod tests {
    use super::*;

    // 目視で確認する
    #[test]
    fn k_means_test() {
        for i in 6..=19 {
            let mut image = open(format!("./problems/{i}.png")).unwrap();
            let samples = k_means_color_sampling(&image, 5, 8, &mut rand::thread_rng());
            replace_pixels_to_nearest_samples(&mut image, &samples);
            image.save(format!("/tmp/{i}.png")).unwrap();
        }
    }
}
*/

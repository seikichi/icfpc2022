mod isl;
mod image;

fn main() {
    let a = isl::Block(vec![0]);
    let b = isl::Block(vec![0]);
    let m = isl::Move::Swap { a, b };
    let program = vec![m];

    println!("Hello, Move: {:?}", program);

    let image = image::Image(vec![vec![isl::Color { r: 0.0, g: 0.0, b: 0.0, a: 0.0 }]]);
    println!("Hello, Image: {:?}", image);
}

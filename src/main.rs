mod isl;
mod image;

fn main() {
    let a = isl::Block(vec![0]);
    let b = isl::Block(vec![0]);
    let m = isl::Move::Swap { a, b };
    let program = vec![m];

    println!("Hello, Move: {:?}", program);

    let img = image::open("problems/1.png");
    for i in 0..10 {
        println!("{:?}", img.0[0][i]);
    }
}

mod isl;

// Image ? Input type

fn main() {
    let a = isl::Block(vec![0]);
    let b = isl::Block(vec![0]);
    let m = isl::Move::Swap { a, b };
    let program = vec![m];

    println!("Hello, Move: {:?}", program);
}

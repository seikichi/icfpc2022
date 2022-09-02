mod isl;
mod image;
mod ai;

fn main() {
    let img = image::open("problems/1.png");
    let solver = ai::OneColorAI {};
    let program = solver.solve(&img);
    println!("{program:?}");
}

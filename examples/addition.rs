use tissue::{run, Input};

fn main() {
    run(
        |x: Vec<f32>| x.iter().sum(),
        &[
            Input::Number(234.289),
            Input::Number(235.6),
        ],
    )
    .expect("Could not run");
}
# Tissue

Tissue is a Rust framework that enables effortless and efficient conversion of machine learning models into interactive, user-friendly demos. With Tissue, a few lines of code are all it takes to bring your machine learning algorithms to life with engaging visual applications.

## Features

- **Speedy Setup**: Get started with Tissue in minutes and integrate seamlessly with your existing Rust machine learning projects.
- **Interactive GUI**: Create a graphical user interface that makes your models accessible to non-technical users.
- **Rust-Powered**: Take advantage of Rust's performance and safety features to deploy machine learning models efficiently.

## Quick Start

To begin using Tissue, add it to your project's `Cargo.toml` file:

```toml
[dependencies]
tissue = "0.1.0"
```

Create an interactive demo with Tissue:

```rust
use tissue::{run, Input};

fn main() {
    run(
        |x: Vec<f32>| x.iter().sum(),
        &[Input::Number(234.289), Input::Number(235.6)],
    )
    .expect("Could not run");
}
```

## Licenses

This library is licensed under either of:

* MIT license [LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT
* Apache License 2.0 [LICENSE-APACHE](LICENSE-APACHE) or https://opensource.org/licenses/Apache-2.0

at your option.

## Acknowledgements

Tissue owes much to the foundational work of [Chris McComb](https://twitter.com/ccmccomb)'s [tease](https://github.com/cmccomb/tease); its initial codebase was critical to Tissue's early development, despite tease no longer being actively maintained. 

Inspired by the user-friendly interfaces of [Gradio](https://gradio.app/) and [Streamlit](https://streamlit.io/), Tissue aspires to streamline the sharing and demonstration of machine learning models within the Rust ecosystem, emulating the simplicity these tools offer.

---

Embrace the power of Rust and bring the magic of your machine learning models to life with Tissue!

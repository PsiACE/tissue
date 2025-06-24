//! # Tissue
//!
//! Tissue is a Rust framework that enables effortless and efficient conversion of machine learning models
//! into interactive, user-friendly demos. With Tissue, a few lines of code are all it takes to bring your
//! machine learning algorithms to life with engaging visual applications.
//!
//! ## Features
//!
//! - **Speedy Setup**: Get started with Tissue in minutes and integrate seamlessly with your existing Rust machine learning projects.
//! - **Interactive GUI**: Create a graphical user interface that makes your models accessible to non-technical users.
//! - **Rust-Powered**: Take advantage of Rust's performance and safety features to deploy machine learning models efficiently.
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use tissue::{run, Input};
//!
//! run(
//!     |x: Vec<f32>| x.iter().sum(),
//!     &[Input::Number(234.289), Input::Number(235.6)],
//! )
//! .expect("Could not run");
//! ```

use std::{
    fmt::{Debug, Display},
    str::FromStr,
};

use num_traits::Float;
use tao::{
    event::{Event, StartCause, WindowEvent},
    event_loop::{ControlFlow, EventLoopBuilder},
    window::WindowBuilder,
};
use wry::WebViewBuilder;

/// Internal event types for communication between the GUI and the prediction logic
enum UserEvents {
    ExecEval(String),
}

/// Represents different types of input controls available in the GUI
#[derive(Debug, Clone)]
pub enum Input {
    /// A numeric input field
    ///
    /// # Example
    /// ```rust
    /// use tissue::Input;
    /// let input = Input::Number(42.0);
    /// ```
    Number(f64),

    /// A text input field
    ///
    /// # Example
    /// ```rust
    /// use tissue::Input;
    /// let input = Input::Text("Hello, World!".to_string());
    /// ```
    Text(String),

    /// A slider input with configurable range and step
    ///
    /// # Example
    /// ```rust
    /// use tissue::Input;
    /// let input = Input::Slider {
    ///     min: 0.0,
    ///     max: 100.0,
    ///     step: 1.0,
    ///     initial_value: 50.0,
    /// };
    /// ```
    Slider {
        /// Minimum value of the slider
        min: f64,
        /// Maximum value of the slider
        max: f64,
        /// Step size for the slider
        step: f64,
        /// Initial value when the interface loads
        initial_value: f64,
    },
}

/// Runs an interactive demo with the given predictor function and input configuration
///
/// This is the main entry point for creating interactive demos. It takes a prediction function
/// that operates on a vector of numeric values and a slice of input configurations.
///
/// # Type Parameters
///
/// - `F`: The numeric type used for calculations (must implement Float, Display, and FromStr)
/// - `P`: The predictor function type
///
/// # Arguments
///
/// - `predictor`: A function that takes a `Vec<F>` and returns a single `F` value
/// - `inputs`: A slice of `Input` configurations that define the GUI controls
///
/// # Returns
///
/// Returns a `wry::Result<()>` indicating success or failure of the GUI creation
///
/// # Errors
///
/// This function will return an error if:
/// - The GUI window cannot be created
/// - The WebView cannot be initialized
/// - There are issues with the event loop
///
/// # Examples
///
/// ## Simple Addition
/// ```rust,no_run
/// use tissue::{run, Input};
///
/// run(
///     |x: Vec<f32>| x.iter().sum(),
///     &[Input::Number(1.0), Input::Number(2.0)],
/// ).expect("Failed to run demo");
/// ```
///
/// ## Linear Regression
/// ```rust,no_run
/// use tissue::{run, Input};
///
/// run(
///     |x: Vec<f32>| {
///         let input_x = x[0];
///         let slope = x[1];
///         let intercept = x[2];
///         slope * input_x + intercept
///     },
///     &[
///         Input::Number(5.0),
///         Input::Slider { min: -10.0, max: 10.0, step: 0.1, initial_value: 2.0 },
///         Input::Slider { min: -10.0, max: 10.0, step: 0.1, initial_value: 1.0 },
///     ],
/// ).expect("Failed to run demo");
/// ```
pub fn run<F, P>(predictor: P, inputs: &[Input]) -> wry::Result<()>
where
    F: 'static + Float + Display + FromStr,
    P: 'static + Fn(Vec<F>) -> F,
    <F as FromStr>::Err: Debug,
{
    // Validate inputs
    if inputs.is_empty() {
        eprintln!("Warning: No inputs provided. The demo will have no interactive controls.");
    }

    // Generate HTML for the interface
    let mut html = beginning();
    for (idx, input) in inputs.iter().enumerate() {
        html.push_str(&match input {
            Input::Number(iv) => add_number(idx, iv),
            Input::Text(s) => add_text(idx, s),
            Input::Slider {
                min,
                max,
                step,
                initial_value,
            } => add_slider(idx, initial_value, max, min, step),
        });
    }
    html.push_str(&end());

    // Set up event loop and IPC communication
    let event_loop = EventLoopBuilder::<UserEvents>::with_user_event().build();
    let proxy = event_loop.create_proxy();

    let ipc_handler = move |req: wry::http::Request<String>| {
        let body = req.body();
        println!("Received IPC message: {}", body);
        let _ = proxy.send_event(UserEvents::ExecEval(body.clone()));
    };

    // Create the main window
    let window = WindowBuilder::new()
        .with_title("Tissue")
        .build(&event_loop)
        .unwrap();

    // Platform-specific WebView setup
    #[cfg(any(
        target_os = "windows",
        target_os = "macos",
        target_os = "ios",
        target_os = "android"
    ))]
    let builder = WebViewBuilder::new();

    #[cfg(target_os = "linux")]
    use tao::platform::unix::WindowExtUnix;
    #[cfg(target_os = "linux")]
    use wry::WebViewBuilderExtUnix;
    #[cfg(not(any(
        target_os = "windows",
        target_os = "macos",
        target_os = "ios",
        target_os = "android"
    )))]
    let builder = {
        let vbox = window.default_vbox().unwrap();
        WebViewBuilder::new_gtk(vbox)
    };

    // Build the WebView
    let _webview = builder
        .with_html(&html)
        .with_ipc_handler(ipc_handler)
        .with_devtools(true)
        .build(&window)
        .unwrap();

    // Main event loop
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::UserEvent(UserEvents::ExecEval(req)) => {
                // Parse input values from the request
                let number_strings = req.split(',');
                let mut inputs = Vec::new();

                for number_str in number_strings {
                    match number_str.trim().parse() {
                        Ok(value) => inputs.push(value),
                        Err(e) => {
                            eprintln!("Failed to parse input '{}': {:?}", number_str, e);
                            return;
                        }
                    }
                }

                // Execute the prediction
                let y = predictor(inputs);

                // Update the output in the UI
                let script = format!("document.getElementById('output').value = {}", y);
                if let Err(e) = _webview.evaluate_script_with_callback(&script, |result| {
                    println!("Script evaluation result: {}", result);
                }) {
                    eprintln!("Failed to evaluate script: {:?}", e);
                }
            }
            Event::NewEvents(StartCause::Init) => {
                println!("Tissue demo started successfully!");
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *control_flow = ControlFlow::Exit,
            _ => (),
        }
    });
}

fn add_input(index: usize, input_type: &str, initial_value: &str) -> String {
    format!("\
    <label for=\"exampleInput{index}\" class=\"col-sm-2 col-form-label mt-3\"><i>x<sub>{index}</sub> = </i></label>\
    <div class=\"col-sm-10 mt-3\">\
        <input type=\"{input_type}\" class=\"form-control input\" id=\"exampleInput{index}\" name=\"x{index}\" aria-describedby=\"input6\" placeholder=\"x{index}\" value=\"{initial_value}\">\
    </div>")
}

fn add_number(index: usize, initial_value: &f64) -> String {
    add_input(index, "text", &initial_value.to_string())
}

fn add_text(index: usize, initial_value: &str) -> String {
    add_input(index, "text", initial_value)
}

fn add_slider(index: usize, initial_value: &f64, max: &f64, min: &f64, step: &f64) -> String {
    format!("\
    <label for=\"exampleInput{index}\" class=\"col-sm-2 col-form-label mt-3\"><i>x<sub>{index}</sub> = </i></label>\
    <div class=\"col-sm-10 mt-3 form-group\" style=\"display: flex\">\
            <input type=\"text\" class=\"form-control col-sm-2\" value=\"{initial_value}\" readonly>
            <input type=\"range\" class=\"form-control input col-sm-10\" min=\"{min}\" max=\"{max}\" step=\"{step}\" id=\"exampleInput{index}\" name=\"x{index}\" aria-describedby=\"input6\" placeholder=\"x{index}\" value=\"{initial_value}\" oninput=\"this.previousElementSibling.value = this.value\">\
    </div>")
}

fn beginning() -> String {
    "<html lang=\"en\">
        <head>
            <meta charset=\"utf-8\">
            <meta name=\"viewport\" content=\"width=device-width, initial-scale=1, shrink-to-fit=no\">
            <link rel=\"stylesheet\" href=\"https://cdn.jsdelivr.net/npm/bootstrap@4.3.1/dist/css/bootstrap.min.css\" integrity=\"sha384-ggOyR0iXCbMQv3Xipma34MD+dH/1fQ784/j6cY/iJTQUOhcWr7x9JvoRxT2MZw1T\" crossorigin=\"anonymous\">
            <script type=\"text/javascript\">
                function run_calculation(event) {
                    event.preventDefault(); // Disables the default form submission behavior
                    var numbers = [];
                    var classes = document.getElementsByClassName('input');
                    Array.from(classes).forEach((x, i) => numbers.push(Number(x.value)));
                    console.log('Sending numbers:', numbers.toString());
                    window.ipc.postMessage(numbers.toString());
                    return false;
                }
            </script>
        </head>
        <body>
            <div class=\"container\">
                <div class=\"row mt-3\">
                    <div class=\"col text-center\">
                        <form action=\"#\" method=\"POST\" onsubmit=\"return run_calculation(event)\">
                            <div class=\"form-group row\" id=\"input-group\">".to_string()
}

fn end() -> String {
    "                        </div>         
        
                            <div class=\"form-group\" id=\"submit\">
                                <button type=\"submit\" class=\"btn btn-primary\">Submit</button>
                            </div>
                        
                            <div class=\"form-group row\" id=\"output-group\">
                                <label for=\"output\" class=\"col-sm-2 col-form-label\"><i>y = </i></label>
                                <div class=\"col-sm-10\">
                                    <input type=\"text\" class=\"form-control\" id=\"output\" name=\"output\" aria-describedby=\"output\" readonly>
                                </div>
                            </div>
                        </form>
                    </div>
                </div>  
            </div>
            <script src=\"https://code.jquery.com/jquery-3.3.1.slim.min.js\" integrity=\"sha384-q8i/X+965DzO0rT7abK41JStQIAqVgRVzpbzo5smXKp4YfRvH+8abtTE1Pi6jizo\" crossorigin=\"anonymous\"></script>
            <script src=\"https://cdn.jsdelivr.net/npm/popper.js@1.14.7/dist/umd/popper.min.js\" integrity=\"sha384-UO2eT0CpHqdSJQ6hJty5KVphtPhzWj9WO1clHTMGa3JDZwrnQq4sF86dIHNDz0W1\" crossorigin=\"anonymous\"></script>
            <script src=\"https://cdn.jsdelivr.net/npm/bootstrap@4.3.1/dist/js/bootstrap.min.js\" integrity=\"sha384-JjSmVgyd0p3pXB1rRibZUAYoIIy6OrQ6VrjIEaFf/nJGzIxFDsf4x0xIM+B07jRM\" crossorigin=\"anonymous\"></script>
        </body>
    </html>".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_input_creation() {
        let number_input = Input::Number(42.0);
        let text_input = Input::Text("test".to_string());
        let slider_input = Input::Slider {
            min: 0.0,
            max: 100.0,
            step: 1.0,
            initial_value: 50.0,
        };

        // Test that inputs can be created without panicking
        match number_input {
            Input::Number(val) => assert_eq!(val, 42.0),
            _ => panic!("Expected Number input"),
        }

        match text_input {
            Input::Text(val) => assert_eq!(val, "test"),
            _ => panic!("Expected Text input"),
        }

        match slider_input {
            Input::Slider {
                min,
                max,
                step,
                initial_value,
            } => {
                assert_eq!(min, 0.0);
                assert_eq!(max, 100.0);
                assert_eq!(step, 1.0);
                assert_eq!(initial_value, 50.0);
            }
            _ => panic!("Expected Slider input"),
        }
    }

    #[test]
    fn test_html_generation() {
        let number_html = add_number(0, &42.0);
        assert!(number_html.contains("42"));
        assert!(number_html.contains("exampleInput0"));
        assert!(number_html.contains("x<sub>0</sub>"));

        let text_html = add_text(1, "test");
        assert!(text_html.contains("test"));
        assert!(text_html.contains("exampleInput1"));
        assert!(text_html.contains("x<sub>1</sub>"));

        let slider_html = add_slider(2, &50.0, &100.0, &0.0, &1.0);
        assert!(slider_html.contains("50"));
        assert!(slider_html.contains("100"));
        assert!(slider_html.contains("0"));
        assert!(slider_html.contains("1"));
        assert!(slider_html.contains("exampleInput2"));
        assert!(slider_html.contains("range"));
    }

    #[test]
    fn test_html_structure() {
        let beginning_html = beginning();
        assert!(beginning_html.contains("<html"));
        assert!(beginning_html.contains("bootstrap"));
        assert!(beginning_html.contains("run_calculation"));
        assert!(beginning_html.contains("ipc.postMessage"));

        let end_html = end();
        assert!(end_html.contains("</html>"));
        assert!(end_html.contains("Submit"));
        assert!(end_html.contains("output"));
        assert!(end_html.contains("jquery"));
    }

    #[test]
    fn test_input_validation() {
        // Test that slider constraints are reasonable
        let slider = Input::Slider {
            min: 0.0,
            max: 100.0,
            step: 1.0,
            initial_value: 50.0,
        };

        if let Input::Slider {
            min,
            max,
            initial_value,
            ..
        } = slider
        {
            assert!(min <= initial_value);
            assert!(initial_value <= max);
            assert!(min < max);
        }
    }

    #[test]
    fn test_input_clone() {
        let original = Input::Number(42.0);
        let cloned = original.clone();

        match (original, cloned) {
            (Input::Number(a), Input::Number(b)) => assert_eq!(a, b),
            _ => panic!("Clone should preserve input type and value"),
        }
    }

    #[test]
    fn test_input_debug() {
        let input = Input::Number(42.0);
        let debug_str = format!("{:?}", input);
        assert!(debug_str.contains("Number"));
        assert!(debug_str.contains("42"));
    }
}

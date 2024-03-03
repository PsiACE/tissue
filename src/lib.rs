use std::{
    cell::RefCell,
    collections::HashMap,
    fmt::{Debug, Display},
    str::FromStr,
};

use num_traits::{zero, Float};
use tao::{
    event::{Event, StartCause, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
use wry::{WebView, WebViewAttributes, WebViewBuilder};

pub enum Input {
    Number(f64),
    Text(String),
    Slider {
        min: f64,
        max: f64,
        step: f64,
        initial_value: f64,
    },
}

pub fn run<F, P>(predictor: P, inputs: &[Input]) -> wry::Result<()>
where
    F: 'static + Float + Display + FromStr,
    P: 'static + Fn(Vec<F>) -> F,
    <F as FromStr>::Err: Debug,
{
    thread_local! {
        static WEBVIEW: RefCell<HashMap<usize, WebView>> = RefCell::new(HashMap::new());
    }

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

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("Tissue")
        .build(&event_loop)
        .unwrap();

    #[cfg(any(
        target_os = "windows",
        target_os = "macos",
        target_os = "ios",
        target_os = "android"
    ))]
    let mut builder = WebViewBuilder::new(&window);

    #[cfg(not(any(
        target_os = "windows",
        target_os = "macos",
        target_os = "ios",
        target_os = "android"
    )))]
    use tao::platform::unix::WindowExtUnix;
    use wry::WebViewBuilderExtUnix;
    let mut builder = {
        let vbox = window.default_vbox().unwrap();
        WebViewBuilder::new_gtk(vbox)
    };

    let mut webview_settings = WebViewAttributes::default();
    webview_settings.devtools = true;
    builder.attrs = webview_settings;
    let _webview = builder
        .with_html(&html)
        .with_ipc_handler(move |req: String| {
            let number_strings = req.split(",");
            let mut inputs = vec![zero(); 0];
            for number in number_strings {
                inputs.push(number.parse().unwrap());
            }

            let y = predictor(inputs);
            WEBVIEW
                .with(|webview| {
                    let webview = webview.borrow();
                    let my_webview = webview.get(&0).unwrap();
                    my_webview.evaluate_script_with_callback(&*format!(
                        "document.getElementById('output').value = {}",
                        y
                    ), |result| println!("{}", result))
                })
                .expect("TODO: panic message");
        })
        .build()
        .unwrap();

    WEBVIEW.with(|wv| {
        let mut hash = wv.borrow_mut();
        hash.insert(0_usize, _webview);
    });

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::NewEvents(StartCause::Init) => println!("Wry has started!"),
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
                function run_calculation() {
                    var numbers = [];
                    var classes = document.getElementsByClassName('input');
                    Array.from(classes).forEach((x, i) => numbers.push(Number(x.value)));
                    ipc.postMessage(numbers.toString());
                }
            </script>
        </head>
        <body>
            <div class=\"container\">
                <div class=\"row mt-3\">
                    <div class=\"col text-center\">
                        <form action=\"#\" method=\"POST\" onsubmit=\"run_calculation()\">
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

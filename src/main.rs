// mod file_manager;
mod renderer;

use std::io::Write;

use renderer::{Line, Renderer};
use termion::raw::IntoRawMode;

fn main() {
    let stdout = std::io::stdout().into_raw_mode().unwrap();
    stdout.lock().flush().unwrap();
    let mut renderer = Renderer::new();
    for i in 0..153 {
        renderer.add_line(Line::new(i.to_string() + " Test Line "));
    }

    loop {
        renderer.render().unwrap();
        if !renderer.update().unwrap() {
            break;
        }
    }
}

// mod file_manager;
mod renderer;

use renderer::*;

fn main() {
    let mut renderer = Renderer::new();
    for i in 0..153 {
        renderer.add_line(Line::new(i.to_string() + " Test Line "));
    }

    println!("{:?}", renderer);
    loop {
        renderer.render().unwrap();
        renderer.update().unwrap();
    }
}

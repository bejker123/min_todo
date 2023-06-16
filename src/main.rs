// mod file_manager;
mod renderer;

use renderer::*;

fn main() {
    let mut renderer = Renderer::new();
    renderer.add_block(Block::new(
        String::from("Test Block 1"),
        10,
        10,
        vec![Line {
            content: String::from("Test line"),
        }],
    ));
    renderer.add_block(Block::new(
        String::from("Test Block 2"),
        10,
        10,
        vec![
            Line {
                content: String::from("1 A really long test line: Lorem ipsum"),
            },
            Line {
                content: String::from("2 A really long test line: Lorem ipsum"),
            },
            Line {
                content: String::from("3 A really long test line: Lorem ipsum"),
            },
            Line {
                content: String::from("4 A really long test line: Lorem ipsum"),
            },
            Line {
                content: String::from("5 A really long test line: Lorem ipsum"),
            },
            Line {
                content: String::from("6 A really long test line: Lorem ipsum"),
            },
            Line {
                content: String::from("7 A really long test line: Lorem ipsum"),
            },
            Line {
                content: String::from("8 A really long test line: Lorem ipsum"),
            },
            Line {
                content: String::from("9 A really long test line: Lorem ipsum"),
            },
            Line {
                content: String::from("10 A really long test line: Lorem ipsum"),
            },
            Line {
                content: String::from("11 A really long test line: Lorem ipsum"),
            },
            Line {
                content: String::from("12 A really long test line: Lorem ipsum"),
            },
            Line {
                content: String::from("13 A really long test line: Lorem ipsum"),
            },
        ],
    ));

    loop {
        renderer.render().unwrap();
        renderer.update().unwrap();
    }
}

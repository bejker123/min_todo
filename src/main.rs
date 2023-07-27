// mod file_manager;
mod char_parser;
mod command_parser;
mod cursor;
mod min_todo;

use std::io::Write;

use min_todo::{Line, MinTodo};
use termion::raw::IntoRawMode;

fn main() {
    print!("{}", termion::screen::ToAlternateScreen);
    let stdout = std::io::stdout().into_raw_mode().unwrap();
    stdout.lock().flush().unwrap();
    let mut renderer = MinTodo::new();
    for i in 0..153 {
        renderer.add_line(Line::new(i.to_string() + " Test Line "));
    }

    loop {
        renderer.render().unwrap();
        if !renderer.update().unwrap() {
            break;
        }
    }
    print!("{}", termion::screen::ToMainScreen);
    stdout.lock().flush().unwrap();
}

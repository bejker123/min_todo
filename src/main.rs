// mod file_manager;
mod char_parser;
mod command_parser;
mod cursor;
mod min_todo;

use std::io::Write;

use min_todo::{Line, MinTodo};
use termion::raw::IntoRawMode;

#[tokio::main]
async fn main() {
    print!("{}", termion::screen::ToAlternateScreen);
    let stdout = std::io::stdout().into_raw_mode().unwrap();
    stdout.lock().flush().unwrap();
    let mut renderer = MinTodo::new();
    for i in 0..153 {
        renderer
            .add_line(Line::from(i.to_string() + " Test Line "))
            .await;
    }

    loop {
        renderer.render().await.unwrap();
        if !renderer.update().await.unwrap() {
            break;
        }
    }
    print!("{}", termion::screen::ToMainScreen);
    stdout.lock().flush().unwrap();
}

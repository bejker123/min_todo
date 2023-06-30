pub type Buffer = [u8; 4];

use crate::command_parser::Command;
use std::{
    error::Error,
    io::{self, Read, Write},
};

use crate::command_parser::CommandParser;

#[derive(Debug)]
pub struct Line {
    pub content: String,
    pub width: usize,
}

impl Line {
    pub fn new<T: std::fmt::Display>(content: T) -> Self {
        Self {
            content: content.to_string(),
            width: content.to_string().len(),
        }
    }

    pub fn len(&self) -> usize {
        self.content.len()
    }

    pub fn render(&self) {
        // let width = self.width.min(self.len());
        print!("\n\r{}", self.content);
        // print!("\n\r{}", &self.content[..width]);
    }
}

//Represent current cursor position (0,0 based)
#[derive(Debug)]
struct Cursor {
    x: usize,
    y: usize,
}

impl Cursor {
    pub fn render(&self, mode: &InputMode) -> Result<(), Box<dyn Error>> {
        print!(
            "{}{}{}",
            termion::cursor::Show.to_string(),
            termion::cursor::Goto(self.x as u16 + 1, self.y as u16 + 1), //We have to add 1, bcs
            //GoTo() is 1,1 based, but
            //we represent the coords
            //as 0,0 based
            match mode {
                InputMode::Normal => termion::cursor::SteadyBlock.to_string(),
                InputMode::Insert => termion::cursor::SteadyBar.to_string(),
            },
        );
        Renderer::flush()?;
        Ok(())
    }

    pub fn move_x(&mut self, x: i32) {
        if x.is_negative() {
            if self.x > 0 {
                self.x -= x.wrapping_abs() as usize;
            }
        } else {
            self.x += x.wrapping_abs() as usize;
        }
    }
    pub fn move_y(&mut self, y: i32) {
        if y.is_negative() {
            if self.y > 0 {
                self.y -= y.wrapping_abs() as usize;
            }
        } else {
            self.y += y.wrapping_abs() as usize;
        }
    }
}

impl Default for Cursor {
    fn default() -> Self {
        Self { x: 0, y: 0 }
    }
}

#[derive(Debug)]
enum InputMode {
    Normal,
    Insert,
}

#[derive(Debug)]
pub struct Renderer {
    content: Vec<Line>,
    cursor: Cursor,
    changed: bool,
    scroll_beg: usize,
    scroll_end: usize,
    start_scroll_up: usize,
    start_scroll_down: usize,
    term_columns: usize,
    command_parser: CommandParser,
    mode: InputMode,
}

impl Renderer {
    pub fn new() -> Self {
        let term_columns = termion::terminal_size().unwrap().1 as usize;
        Self {
            content: Vec::new(),
            cursor: Cursor::default(),
            changed: true,
            scroll_beg: 0,
            scroll_end: term_columns,
            term_columns,
            start_scroll_up: 5,
            start_scroll_down: term_columns - 5,
            command_parser: CommandParser::new(),
            mode: InputMode::Normal,
        }
    }

    pub fn add_line(&mut self, line: Line) {
        self.content.push(line);
        // self.scroll_end += 1;
    }

    //Clear and move the cursor to (1,1)
    fn clear() {
        print!("{}{}", termion::cursor::Goto(1, 1), termion::clear::All);
    }

    fn flush() -> Result<(), Box<dyn Error>> {
        std::io::stdout().flush()?;

        Ok(())
    }

    fn exit() -> bool {
        Self::clear();
        let _ = Self::flush(); //Here we don't care if we succed or not.
        false
    }

    fn move_cur_down(&mut self) {
        if self.cursor.y >= self.start_scroll_down && self.scroll_end < self.content.len() {
            if self.scroll_beg < self.scroll_end {
                self.scroll_beg += 1;
            }
            self.scroll_end += 1;
        } else {
            if self.get_current_line() < self.content.len() - 1 {
                self.cursor.move_y(1);
            }
        }
    }
    fn move_cur_up(&mut self) {
        if self.cursor.y == self.start_scroll_up && self.scroll_beg > 0
            || (self.cursor.y == 0 && self.scroll_beg != 0)
        {
            self.scroll_beg -= 1;
            if self.scroll_end > 0 {
                self.scroll_end -= 1;
            }
        } else {
            self.cursor.move_y(-1);
        }
    }

    fn move_to_line(&mut self, line: usize) {
        if line >= self.content.len() - 1 {
            self.move_to_bottom();
            return;
        }

        self.scroll_beg = line;
        self.scroll_end = self.scroll_beg + self.term_columns;
        self.cursor.y = 0; //self.term_columns as i32 / 2;
    }

    fn move_to_top(&mut self) {
        self.scroll_beg = 0;
        self.scroll_end = self.term_columns;
        self.cursor.y = 0;
    }

    fn move_to_bottom(&mut self) {
        self.scroll_end = self.content.len();
        self.scroll_beg = self.scroll_end - self.term_columns;
        self.cursor.y = self.term_columns - 1;
    }

    fn get_current_line(&self) -> usize {
        self.cursor.y + self.scroll_beg
    }

    //Return false to exit.
    fn handle_character(&mut self, buffer: Buffer) -> bool {
        match self.mode {
            InputMode::Normal => {
                if let Some(command) = self
                    .command_parser
                    .parse_command(*buffer.first().unwrap() as char)
                {
                    match command {
                        Command::Quit => return Self::exit(),
                        Command::MoveDown => self.move_cur_down(),
                        Command::MoveUp => self.move_cur_up(),
                        Command::MoveLeft => self.cursor.move_x(-1),
                        Command::MoveRight => self.cursor.move_x(1),
                        Command::MoveToBottom => {
                            if let Some(nr_prefix) = self.command_parser.nr_prefix() {
                                self.move_to_line(nr_prefix);
                                self.command_parser.clear_nr_prefix();
                            } else {
                                self.move_to_bottom();
                            }
                        }
                        Command::MoveToTop => {
                            if let Some(nr_prefix) = self.command_parser.nr_prefix() {
                                self.move_to_line(nr_prefix);
                                self.command_parser.clear_nr_prefix();
                            } else {
                                self.move_to_top();
                            }
                        }
                        Command::EnterInsertMode => {
                            self.mode = InputMode::Insert;
                            self.changed = true;
                        }
                        Command::Append => {
                            self.mode = InputMode::Insert;
                            self.cursor.move_x(1);
                            self.changed = true;
                        }

                        _ => {}
                    }
                }
            }
            InputMode::Insert => match buffer {
                //Esc
                [27, 0, 0, 0] => {
                    self.mode = InputMode::Normal;
                    self.cursor.move_x(-1);
                    self.changed = true;
                }
                //Backspace
                [127, 0, 0, 0] => {
                    if self.cursor.x != 0 {
                        let curr = self.get_current_line();
                        if let Some(curr) = self.content.get_mut(curr) {
                            if self.cursor.x - 1 < curr.len() {
                                curr.content.remove(self.cursor.x - 1);
                                self.cursor.move_x(-1);
                            }
                        }
                    }
                }
                //Delete
                [27, 91, 51, 126] => {
                    let curr = self.get_current_line();
                    if let Some(curr) = self.content.get_mut(curr) {
                        if self.cursor.x < curr.len() {
                            curr.content.remove(self.cursor.x);
                        }
                    }
                }
                //Arrow Up
                [27, 91, 65, 0] => {
                    self.move_cur_up();
                }
                //Arrow Down
                [27, 91, 66, 0] => {
                    self.move_cur_down();
                }
                //Arrow Left
                [27, 91, 68, 0] => {
                    self.cursor.move_x(-1);
                }
                //Arrow Left
                [27, 91, 67, 0] => {
                    self.cursor.move_x(1);
                }
                _ => {
                    let curr = self.get_current_line();
                    if let Some(curr) = self.content.get_mut(curr) {
                        curr.content
                            .insert(self.cursor.x, *buffer.first().unwrap() as char);
                        self.cursor.move_x(1);
                    }
                }
            },
        }
        true
    }

    pub fn update(&mut self) -> Result<bool, Box<dyn Error>> {
        // let mut buf = Vec::new();
        // std::io::stdin().(&mut buf)?;
        // println!("{:?}", buf);
        let mut buffer = [0, 0, 0, 0];
        let mut stdin = io::stdin();

        // stdout.lock().flush().unwrap();

        stdin.read(&mut buffer)?;
        // panic!("{:?}", buffer);
        if !self.handle_character(buffer) {
            return Ok(false);
        }
        // println!("{buffer:?}");
        self.changed = true;
        // std::thread::sleep_ms(100);
        Ok(true)
    }

    pub fn render(&mut self) -> Result<(), Box<dyn Error>> {
        if !self.changed {
            return Ok(());
        }
        self.changed = false;
        Self::clear();

        let start = self.scroll_beg;
        // .min(self.scroll_end - START_SCROLL_UP as usize);
        let end = self.scroll_end.min(self.content.len());

        for c in &self.content[start..end] {
            c.render();
        }
        Self::flush()?;
        self.cursor.render(&self.mode)?;
        Ok(())
    }
}

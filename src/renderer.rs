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
        let width = self.width.min(self.len());
        print!("\n\r{}", &self.content[..width]);
    }
}

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
            termion::cursor::Goto(self.x as u16 + 1, self.y as u16 + 1),
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
        if self.cursor.y >= self.start_scroll_down {
            if self.scroll_beg < self.scroll_end {
                self.scroll_beg += 1;
            }
            // if self.scroll_end < self.content.len() {
            self.scroll_end += 1;
            // }
        } else {
            self.cursor.move_y(1);
        }
    }
    fn move_cur_up(&mut self) {
        if self.cursor.y == self.start_scroll_up && self.scroll_beg > 0_usize
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
        self.cursor.y = self.term_columns;
    }

    //Return false to exit.
    fn handle_character(&mut self, c: char) -> bool {
        match self.mode {
            InputMode::Normal => {
                if let Some(command) = self.command_parser.parse_command(c) {
                    match command {
                        Command::Quit => return Self::exit(),
                        Command::MoveDown => self.move_cur_down(),
                        Command::MoveUp => self.move_cur_up(),
                        Command::MoveLeft => self.cursor.move_x(-1),
                        Command::MoveRight => self.cursor.move_x(1),
                        Command::MoveToBottom => {
                            if let Some(nr_prefix) = self.command_parser.nr_prefix() {
                                self.move_to_line(nr_prefix as usize);
                                self.command_parser.clear_nr_prefix();
                            } else {
                                self.move_to_bottom();
                            }
                        }
                        Command::MoveToTop => {
                            if let Some(nr_prefix) = self.command_parser.nr_prefix() {
                                self.move_to_line(nr_prefix as usize);
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
            InputMode::Insert => match c {
                '\u{1B}' => {
                    self.mode = InputMode::Normal;
                    self.cursor.move_x(-1);
                    self.changed = true;
                }
                _ => {}
            },
        }

        // '\u{3}' | 'q' => return Self::exit(),
        // 'j' => self.move_cur_down(),
        // 'k' => self.move_cur_up(),
        // 'h' => self.cursor.move_x(-1),
        // 'l' => self.cursor.move_x(1),
        // 'G' => return Self::exit(),
        true
    }

    pub fn update(&mut self) -> Result<bool, Box<dyn Error>> {
        // let mut buf = Vec::new();
        // std::io::stdin().(&mut buf)?;
        // println!("{:?}", buf);
        let mut buffer = [0];
        let mut stdin = io::stdin();

        // stdout.lock().flush().unwrap();

        stdin.read_exact(&mut buffer)?;
        // println!("{:?}", buffer);
        if let Some(c) = buffer.first() {
            if !self.handle_character(*c as char) {
                return Ok(false);
            }
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
